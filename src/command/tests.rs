use crate::{
    asset_ptr::AssetPtr,
    asset_surface::GroupSurface,
    command::{
        EngineCommand, EngineResponse, MeshPublish,
        OP_ALLOC_MEM, OP_DROP_GROUPS, OP_EXPORT_ALL, OP_EXPORT_ASSETS,
        OP_GET_SURFACE_TYPES, OP_IMPORT_ASSETS,
        OP_ORGANIZE_OBJECTS, OP_SEND_MESH, OP_SET_SURFACE_TYPES,
        OP_STANDARDIZE_GROUPS, OP_STANDARDIZE_SYNCED_GROUPS, OP_STOP_ENGINE,
    },
    fields::Uuid,
};

fn make_uuid(id: u8) -> Uuid {
    let mut bytes = [0u8; Uuid::SIZE];
    bytes[0] = id;
    Uuid { bytes }
}

// ============================================================================
// EngineCommand — simple roundtrip tests
// ============================================================================

#[test]
fn test_send_mesh_roundtrip() {
    let ptrs = [AssetPtr::new(0, 100), AssetPtr::new(1, 200), AssetPtr::new(2, 300)];
    let cmd = EngineCommand::send_mesh(&ptrs);
    assert_eq!(cmd.op_id, OP_SEND_MESH);
    assert_eq!(cmd.num_headers, 3);
    let result = cmd.read_send_mesh().unwrap();
    assert_eq!(result, &ptrs);
}

#[test]
fn test_standardize_groups_roundtrip() {
    let uuids = [make_uuid(1), make_uuid(2), make_uuid(3)];
    let cmd = EngineCommand::standardize_groups(&uuids);
    assert_eq!(cmd.op_id, OP_STANDARDIZE_GROUPS);
    let result = cmd.read_standardize_groups().unwrap();
    assert_eq!(result, &uuids);
}

#[test]
fn test_drop_groups_roundtrip() {
    let uuids = [make_uuid(5), make_uuid(6)];
    let cmd = EngineCommand::drop_groups(&uuids, 1);
    assert_eq!(cmd.op_id, OP_DROP_GROUPS);
    assert_eq!(cmd.should_cache, 1);
    let result = cmd.read_drop_groups().unwrap();
    assert_eq!(result, &uuids);
}

#[test]
fn test_standardize_synced_groups_roundtrip() {
    let surfaces = [
        GroupSurface::new(make_uuid(1), 42),
        GroupSurface::new(make_uuid(2), 99),
    ];
    let cmd = EngineCommand::standardize_synced_groups(&surfaces, 1);
    assert_eq!(cmd.op_id, OP_STANDARDIZE_SYNCED_GROUPS);
    let result = cmd.read_standardize_synced_groups().unwrap();
    assert_eq!(result, &surfaces);
}

#[test]
fn test_set_surface_types_roundtrip() {
    let surfaces = [
        GroupSurface::new(make_uuid(3), 1),
        GroupSurface::new(make_uuid(4), 2),
        GroupSurface::new(make_uuid(5), 3),
    ];
    let cmd = EngineCommand::set_surface_types(&surfaces, 1);
    assert_eq!(cmd.op_id, OP_SET_SURFACE_TYPES);
    let result = cmd.read_set_surface_types().unwrap();
    assert_eq!(result, &surfaces);
}

// ============================================================================
// EngineCommand — empty command tests
// ============================================================================

#[test]
fn test_organize_objects_empty() {
    let cmd = EngineCommand::organize_objects(1);
    assert_eq!(cmd.op_id, OP_ORGANIZE_OBJECTS);
    assert_eq!(cmd.should_cache, 1);
    cmd.read_organize_objects();
}

#[test]
fn test_get_surface_types_empty() {
    let cmd = EngineCommand::get_surface_types(1);
    assert_eq!(cmd.op_id, OP_GET_SURFACE_TYPES);
    cmd.read_get_surface_types();
}

#[test]
fn test_stop_engine_empty() {
    let cmd = EngineCommand::stop_engine();
    assert_eq!(cmd.op_id, OP_STOP_ENGINE);
    cmd.read_stop_engine();
}

// ============================================================================
// EngineCommand — complex layout tests
// ============================================================================

#[test]
fn test_alloc_request_roundtrip() {
    let uuids = [make_uuid(1), make_uuid(2)];
    let sizes = [1024usize, 2048];
    let cmd = EngineCommand::alloc_request(&uuids, &sizes);
    assert_eq!(cmd.op_id, OP_ALLOC_MEM);
    let (read_uuids, read_sizes) = cmd.read_alloc_request().unwrap();
    assert_eq!(read_uuids, &uuids);
    assert_eq!(read_sizes, &sizes);
}

#[test]
fn test_alloc_request_mismatch_panics() {
    let uuids = [make_uuid(1)];
    let sizes = [100usize, 200];
    let result = std::panic::catch_unwind(|| {
        EngineCommand::alloc_request(&uuids, &sizes);
    });
    assert!(result.is_err());
}

#[test]
fn test_export_assets_roundtrip() {
    let uuids = [make_uuid(1), make_uuid(2)];
    let cmd = EngineCommand::export_assets("/tmp/export", 1_048_576, &uuids);
    assert_eq!(cmd.op_id, OP_EXPORT_ASSETS);
    let (path, target, read_uuids) = cmd.read_export_assets().unwrap();
    assert_eq!(path, "/tmp/export");
    assert_eq!(target, 1_048_576);
    assert_eq!(read_uuids, &uuids);
}

#[test]
fn test_export_all_roundtrip() {
    let cmd = EngineCommand::export_all("/tmp/batch", 524_288);
    assert_eq!(cmd.op_id, OP_EXPORT_ALL);
    let (path, target) = cmd.read_export_all().unwrap();
    assert_eq!(path, "/tmp/batch");
    assert_eq!(target, 524_288);
}

#[test]
fn test_import_assets_roundtrip() {
    let paths = ["/tmp/a.elbo", "/tmp/b.elbo", "/tmp/c.elbo"];
    let cmd = EngineCommand::import_assets(&paths);
    assert_eq!(cmd.op_id, OP_IMPORT_ASSETS);
    assert_eq!(cmd.num_headers, 3);
    let result = cmd.read_import_assets().unwrap();
    assert_eq!(result.len(), 3);
    assert_eq!(result[0], paths[0]);
    assert_eq!(result[1], paths[1]);
    assert_eq!(result[2], paths[2]);
}

// ============================================================================
// EngineResponse — roundtrip and status tests
// ============================================================================

#[test]
fn test_response_status_defaults() {
    let resp = EngineResponse::send_mesh();
    assert_eq!(resp.read_send_mesh(), 0);

    let resp = EngineResponse::standardize_groups();
    assert_eq!(resp.read_standardize_groups(), 0);

    let resp = EngineResponse::drop_groups();
    assert_eq!(resp.read_drop_groups(), 0);
}

#[test]
fn test_alloc_response_roundtrip() {
    let uuids = [make_uuid(1), make_uuid(2)];
    let ptrs = [AssetPtr::new(0, 100), AssetPtr::new(1, 200)];
    let resp = EngineResponse::alloc_response(&uuids, &ptrs);
    let (read_uuids, read_ptrs) = resp.read_alloc_response().unwrap();
    assert_eq!(read_uuids, &uuids);
    assert_eq!(read_ptrs, &ptrs);
}

#[test]
fn test_export_assets_response_roundtrip() {
    let filenames = ["batch_000.elbo", "batch_001.elbo"];
    let resp = EngineResponse::export_assets(&filenames);
    let result = resp.read_export_assets().unwrap();
    assert_eq!(result, filenames.as_slice());
}

#[test]
fn test_export_all_response_roundtrip() {
    let filenames = ["all_assets.elbo"];
    let resp = EngineResponse::export_all(&filenames);
    let result = resp.read_export_all().unwrap();
    assert_eq!(result, filenames.as_slice());
}

#[test]
fn test_import_assets_response_status() {
    let resp = EngineResponse::import_assets();
    assert_eq!(resp.read_import_assets(), 0);
}

// ============================================================================
// MeshPublish — roundtrip and status tests
// ============================================================================

#[test]
fn test_mesh_publish_send_mesh_roundtrip() {
    let ptrs = [AssetPtr::new(0, 100), AssetPtr::new(1, 200), AssetPtr::new(2, 300)];
    let mesh = MeshPublish::send_mesh(&ptrs);
    assert_eq!(mesh.header.num_items, 3);
    let result = mesh.read_send_mesh().unwrap();
    assert_eq!(result, &ptrs);
}

#[test]
fn test_mesh_publish_send_mesh_empty() {
    let mesh = MeshPublish::send_mesh(&[]);
    assert_eq!(mesh.header.num_items, 0);
    let result = mesh.read_send_mesh().unwrap();
    assert!(result.is_empty());
}

#[test]
fn test_mesh_publish_standardize_groups_status() {
    let mesh = MeshPublish::standardize_groups();
    assert_eq!(mesh.read_standardize_groups(), 0);
}

#[test]
fn test_mesh_publish_organize_objects_status() {
    let mesh = MeshPublish::organize_objects();
    assert_eq!(mesh.read_organize_objects(), 0);
}
