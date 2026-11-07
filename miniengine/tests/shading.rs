//! Analytic shader-unit fixtures. No assertion of native DXR pixel equivalence.
use alkyhol_d3d12::{D3D12CreateDevice, complete};
use metal::*;
#[test]
#[allow(clippy::approx_constant)] // Match upstream 3.14159 literal in the sun formula.
fn metal_tangent_repair_mirrored_uvs_lighting_shadow_endpoints_and_tone_curve() {
    objc::rc::autoreleasepool(|| {
        let device = pollster::block_on(D3D12CreateDevice(false)).unwrap();
        let native = device.metal_device();
        let source = concat!(
            include_str!("../src/render.metal"),
            "\n",
            include_str!("../src/shading.metal"),
            r#"
kernel void shader_contract(device float4* out [[buffer(0)]],uint id [[thread_position_in_grid]]) {
 float3 n=float3(0,0,1),x=float3(1,0,0),y=float3(0,1,0);uint repaired=0;
 if(id<4){float3 t=id==0?x*2:float3(NAN),b=id==0?y*3:float3(NAN);float2 d1=id==2?float2(-1,0):float2(1,0),d2=float2(0,1);if(id==3){d1=0;d2=0;}float3x3 frame=tangent_frame(n,t,b,x,y,d1,d2,repaired);out[id]=float4(frame*float3(0.2,0.3,0.9),float(repaired));}
 if(id==4)out[id]=float4(display_colour(float3(0)),1);
 if(id==5)out[id]=float4(display_colour(float3(0.1,1,4)),1);
 if(id==6){Surface surface={float3(0),sun_direction(),float3(0.5,0.25,0.1),0,0,128,0};out[id]=float4(lit_colour(surface,float3(0,0,-1),0,false),1);}
 if(id==7){Surface surface={float3(0),sun_direction(),float3(0.5,0.25,0.1),0,0,128,0};out[id]=float4(lit_colour(surface,float3(0,0,-1),1,false),1);}
}
"#
        );
        let options = CompileOptions::new();
        options.set_fast_math_enabled(false);
        let library = native.new_library_with_source(source, &options).unwrap();
        let function = library.get_function("shader_contract", None).unwrap();
        let pipeline = native
            .new_compute_pipeline_state_with_function(&function)
            .unwrap();
        let output = native.new_buffer(128, MTLResourceOptions::StorageModeShared);
        let command = device.metal_queue().new_command_buffer();
        let encoder = command.new_compute_command_encoder();
        encoder.set_compute_pipeline_state(&pipeline);
        encoder.set_buffer(0, Some(&output), 0);
        encoder.dispatch_threads(MTLSize::new(8, 1, 1), MTLSize::new(8, 1, 1));
        encoder.end_encoding();
        complete(command).unwrap();
        let values = unsafe { std::slice::from_raw_parts(output.contents().cast::<[f32; 4]>(), 8) };
        let compare = |id: usize, expected: [f32; 4]| {
            for (actual, expected) in values[id].iter().zip(expected) {
                assert!(
                    actual.is_finite() && (actual - expected).abs() < 1e-5,
                    "shader result {}: {:?}, expected component {}",
                    id,
                    values[id],
                    expected
                );
            }
        };
        compare(0, [0.2, 0.3, 0.9, 0.]);
        compare(1, [0.2, 0.3, 0.9, 1.]);
        compare(2, [-0.2, 0.3, 0.9, 1.]);
        compare(3, [0.2, 0.3, 0.9, 1.]);
        compare(4, [0., 0., 0., 1.]);
        let curve = |linear: f64| {
            let hdr = linear * 2.;
            let shoulder = hdr * hdr.sqrt();
            let sdr = shoulder / (shoulder + (4f64 / 27.).sqrt());
            if sdr <= 0.0031308 {
                sdr * 12.92
            } else {
                1.055 * sdr.powf(1. / 2.4) - 0.055
            }
        };
        compare(
            5,
            [curve(0.1) as f32, curve(1.) as f32, curve(4.) as f32, 1.],
        );
        compare(6, [0.05, 0.025, 0.01, 1.]);
        let sun = glam::Vec3::new(
            (-0.5f32).cos() * (0.75f32 * 3.14159 * 0.5).cos(),
            (0.75f32 * 3.14159 * 0.5).sin(),
            (-0.5f32).sin() * (0.75f32 * 3.14159 * 0.5).cos(),
        )
        .normalize();
        let half = (sun + glam::Vec3::new(0., 0., -1.)).normalize();
        let fresnel = (1. - sun.dot(half).clamp(0., 1.)).powi(5);
        compare(
            7,
            [
                0.05 + 4. * 0.5 * (1. - fresnel),
                0.025 + 4. * 0.25 * (1. - fresnel),
                0.01 + 4. * 0.1 * (1. - fresnel),
                1.,
            ],
        );
    });
}
