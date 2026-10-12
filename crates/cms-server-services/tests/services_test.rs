use cms_server_services::{
    AuthAuditService, EventDispatcherService, StorageUrlService, TenantContextService,
};
use uuid::Uuid;

#[test]
fn test_storage_key_formatting() {
    let workspace_id = "ws-123";
    let run_id = Uuid::new_v4();
    let report_key = StorageUrlService::format_report_storage_key(workspace_id, &run_id, "csv");
    assert_eq!(report_key, format!("reports/ws-123/{run_id}.csv"));

    let asset_id = Uuid::new_v4();
    let asset_key = StorageUrlService::format_asset_storage_key(workspace_id, &asset_id, "logo.png");
    assert_eq!(asset_key, format!("assets/ws-123/{asset_id}/logo.png"));
}

#[test]
fn test_service_instantiation() {
    let _ = AuthAuditService;
    let _ = TenantContextService;
    let _ = EventDispatcherService;
}
