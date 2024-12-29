use crate::controllers::health::health_check;

#[tokio::test]
async fn test_health_check() {
    let response = health_check().await;
    let json = response.0;
    assert_eq!(json["status"], "ok");
}
