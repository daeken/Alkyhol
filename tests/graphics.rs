use alkyhol::graphics::*;
use std::collections::HashMap;
fn resources() -> HashMap<u64, ResourceState> {
    HashMap::from([(1, ResourceState::RenderTarget)])
}
#[test]
fn record_close_reset_state_machine() {
    let mut list = GraphicsCommandList::new(7);
    assert_eq!(list.commands(), Err(GraphicsError::ListOpen));
    assert_eq!(list.reset(), Err(GraphicsError::ListOpen));
    list.close().unwrap();
    assert_eq!(list.close(), Err(GraphicsError::ListClosed));
    assert_eq!(
        list.record(Command::Transition {
            resource: 1,
            before: ResourceState::RenderTarget,
            after: ResourceState::CopySource
        }),
        Err(GraphicsError::ListClosed)
    );
    list.reset().unwrap();
    list.close().unwrap();
    assert!(list.commands().unwrap().is_empty());
}
#[test]
fn failed_submission_cannot_advance_resource_state() {
    let original = resources();
    let mut list = GraphicsCommandList::new(7);
    list.record(Command::Transition {
        resource: 1,
        before: ResourceState::RenderTarget,
        after: ResourceState::CopySource,
    })
    .unwrap();
    list.record(Command::ClearRenderTargetView {
        resource: 1,
        color: [1.0; 4],
    })
    .unwrap();
    list.close().unwrap();
    assert_eq!(
        validate_submission(7, &list, &original, &[]),
        Err(GraphicsError::WrongState)
    );
    assert_eq!(original[&1], ResourceState::RenderTarget);
}
#[test]
fn successful_draw_and_transition() {
    let mut list = GraphicsCommandList::new(7);
    list.record(Command::DrawInstanced {
        resource: 1,
        pipeline: 2,
        vertices: 3,
        instances: 1,
    })
    .unwrap();
    list.record(Command::Transition {
        resource: 1,
        before: ResourceState::RenderTarget,
        after: ResourceState::CopySource,
    })
    .unwrap();
    list.close().unwrap();
    assert_eq!(
        validate_submission(7, &list, &resources(), &[2]).unwrap()[&1],
        ResourceState::CopySource
    );
    assert_eq!(
        validate_submission(8, &list, &resources(), &[2]),
        Err(GraphicsError::WrongDevice)
    );
    assert_eq!(
        validate_submission(7, &list, &resources(), &[]),
        Err(GraphicsError::InvalidArgument)
    );
}
#[test]
fn reject_unknown_resource_bad_color_and_unsupported_draw() {
    let mut list = GraphicsCommandList::new(7);
    assert_eq!(
        list.record(Command::ClearRenderTargetView {
            resource: 1,
            color: [f64::NAN; 4]
        }),
        Err(GraphicsError::InvalidArgument)
    );
    list.record(Command::ClearRenderTargetView {
        resource: 9,
        color: [0.0; 4],
    })
    .unwrap();
    list.close().unwrap();
    assert_eq!(
        validate_submission(7, &list, &resources(), &[]),
        Err(GraphicsError::InvalidArgument)
    );
    list.reset().unwrap();
    list.record(Command::DrawInstanced {
        resource: 1,
        pipeline: 2,
        vertices: 6,
        instances: 1,
    })
    .unwrap();
    list.close().unwrap();
    assert_eq!(
        validate_submission(7, &list, &resources(), &[2]),
        Err(GraphicsError::Unsupported)
    );
}
#[test]
fn aligned_readback_and_overflow() {
    assert_eq!(copy_footprint(1, 2).unwrap(), (256, 512));
    assert_eq!(copy_footprint(65, 3).unwrap(), (512, 1536));
    assert_eq!(copy_footprint(0, 1), Err(GraphicsError::InvalidArgument));
    assert_eq!(
        copy_footprint(u32::MAX, 1),
        Err(GraphicsError::InvalidArgument)
    );
}

#[test]
fn recording_rejects_each_invalid_color_without_appending() {
    let mut list = GraphicsCommandList::new(42);
    assert_eq!(list.device(), 42);
    for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY, -0.01, 1.01] {
        for channel in 0..4 {
            let mut color = [0.0; 4];
            color[channel] = value;
            assert_eq!(
                list.record(Command::ClearRenderTargetView { resource: 1, color }),
                Err(GraphicsError::InvalidArgument)
            );
        }
    }
    list.close().unwrap();
    assert!(list.commands().unwrap().is_empty());
}

#[test]
fn submission_requires_closed_list_and_correct_states_and_tokens() {
    let mut list = GraphicsCommandList::new(7);
    assert_eq!(
        validate_submission(7, &list, &resources(), &[]),
        Err(GraphicsError::ListOpen)
    );
    for command in [
        Command::Transition {
            resource: 9,
            before: ResourceState::RenderTarget,
            after: ResourceState::CopySource,
        },
        Command::DrawInstanced {
            resource: 9,
            pipeline: 2,
            vertices: 3,
            instances: 1,
        },
        Command::DrawInstanced {
            resource: 1,
            pipeline: 99,
            vertices: 3,
            instances: 1,
        },
    ] {
        list.record(command).unwrap();
        list.close().unwrap();
        assert_eq!(
            validate_submission(7, &list, &resources(), &[2]),
            Err(GraphicsError::InvalidArgument)
        );
        list.reset().unwrap();
    }
    list.record(Command::DrawInstanced {
        resource: 1,
        pipeline: 2,
        vertices: 3,
        instances: 0,
    })
    .unwrap();
    list.close().unwrap();
    assert_eq!(
        validate_submission(7, &list, &resources(), &[2]),
        Err(GraphicsError::Unsupported)
    );
    let source = HashMap::from([(1, ResourceState::CopySource)]);
    assert_eq!(
        validate_submission(7, &list, &source, &[2]),
        Err(GraphicsError::WrongState)
    );
}

#[test]
fn transitions_are_sequential_and_reset_discards_recorded_commands() {
    let mut list = GraphicsCommandList::new(7);
    for (before, after) in [
        (ResourceState::RenderTarget, ResourceState::CopySource),
        (ResourceState::CopySource, ResourceState::RenderTarget),
    ] {
        list.record(Command::Transition {
            resource: 1,
            before,
            after,
        })
        .unwrap();
    }
    list.record(Command::ClearRenderTargetView {
        resource: 1,
        color: [1.0; 4],
    })
    .unwrap();
    list.close().unwrap();
    assert_eq!(
        validate_submission(7, &list, &resources(), &[]).unwrap(),
        resources()
    );
    list.reset().unwrap();
    list.close().unwrap();
    assert!(list.commands().unwrap().is_empty());
}

#[test]
fn footprint_alignment_boundaries_and_large_products() {
    assert_eq!(copy_footprint(64, 1).unwrap(), (256, 256));
    assert_eq!(copy_footprint(0, 0), Err(GraphicsError::InvalidArgument));
    assert_eq!(copy_footprint(1, 0), Err(GraphicsError::InvalidArgument));
    assert_eq!(
        copy_footprint(u32::MAX / 4, 1),
        Err(GraphicsError::InvalidArgument)
    );
    let width = (u32::MAX - 255) / 4;
    let pitch = u32::MAX - 255;
    assert_eq!(
        copy_footprint(width, u32::MAX).unwrap(),
        (pitch, u64::from(pitch) * u64::from(u32::MAX))
    );
    assert_eq!(GraphicsError::WrongState.to_string(), "WrongState");
}
