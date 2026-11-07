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
