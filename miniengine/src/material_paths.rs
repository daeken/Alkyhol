// Pinned ModelH3D LoadTextures resolutions; None means the original engine default.
pub const MATERIAL_PATHS: [[Option<&str>; 3]; 27] = [
    [None, None, None],
    [
        Some("sponza_thorn.DDS"),
        Some("sponza_thorn_normal.DDS"),
        Some("sponza_thorn_specular.DDS"),
    ],
    [
        Some("vase_round.DDS"),
        Some("vase_round_normal.DDS"),
        Some("vase_round_specular.DDS"),
    ],
    [
        Some("vase_plant.DDS"),
        None,
        Some("vase_plant_specular.DDS"),
    ],
    [Some("background.DDS"), Some("background_normal.DDS"), None],
    [Some("gi_flag.DDS"), None, None],
    [
        Some("spnza_bricks_a.DDS"),
        Some("spnza_bricks_a_normal.DDS"),
        Some("spnza_bricks_a_specular.DDS"),
    ],
    [
        Some("sponza_arch.DDS"),
        Some("sponza_arch_normal.DDS"),
        Some("sponza_arch_specular.DDS"),
    ],
    [
        Some("sponza_ceiling_a.DDS"),
        None,
        Some("sponza_ceiling_a_specular.DDS"),
    ],
    [
        Some("sponza_column_a.DDS"),
        Some("sponza_column_a_normal.DDS"),
        Some("sponza_column_a_specular.DDS"),
    ],
    [
        Some("sponza_floor_a.DDS"),
        None,
        Some("sponza_floor_a_specular.DDS"),
    ],
    [
        Some("sponza_column_c.DDS"),
        Some("sponza_column_c_normal.DDS"),
        Some("sponza_column_c_specular.DDS"),
    ],
    [
        Some("sponza_details.DDS"),
        None,
        Some("sponza_details_specular.DDS"),
    ],
    [
        Some("sponza_column_b.DDS"),
        Some("sponza_column_b_normal.DDS"),
        Some("sponza_column_b_specular.DDS"),
    ],
    [Some("white.DDS"), None, None],
    [
        Some("sponza_flagpole.DDS"),
        None,
        Some("sponza_flagpole_specular.DDS"),
    ],
    [Some("sponza_fabric_green.DDS"), None, None],
    [Some("sponza_fabric_blue.DDS"), None, None],
    [
        Some("sponza_fabric.DDS"),
        None,
        Some("sponza_fabric_specular.DDS"),
    ],
    [Some("sponza_curtain_blue.DDS"), None, None],
    [Some("sponza_curtain.DDS"), None, None],
    [Some("sponza_curtain_green.DDS"), None, None],
    [
        Some("chain_texture.DDS"),
        Some("chain_texture_normal.DDS"),
        None,
    ],
    [Some("vase_hanging.DDS"), None, None],
    [Some("vase.DDS"), Some("vase_normal.DDS"), None],
    [Some("lion.DDS"), Some("lion_normal.DDS"), None],
    [Some("sponza_roof.DDS"), None, None],
];
