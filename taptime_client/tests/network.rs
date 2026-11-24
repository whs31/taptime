use std::io::{Read, Write};

use taptime_client::{ApiError, Session, TapTimeClient};

#[tokio::test]
async fn unauthorized_network_failure_and_recovery() {
  let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
  let endpoint = format!("http://{}", listener.local_addr().unwrap());
  let server = std::thread::spawn(move || {
    for status in [401, 503, 401] {
      let (mut stream, _) = listener.accept().unwrap();
      stream
        .set_read_timeout(Some(std::time::Duration::from_secs(5)))
        .unwrap();
      let mut bytes = [0; 8192];
      let _ = stream.read(&mut bytes).unwrap();
      write!(
        stream,
        "HTTP/1.1 {status} Test\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
      )
      .unwrap();
    }
  });
  let client = TapTimeClient::new(&endpoint).unwrap();
  let session = Session::new("fixture-token").unwrap();
  assert!(matches!(
    client.get_user(&session).await,
    Err(ApiError::Unauthenticated)
  ));
  assert!(matches!(
    client.get_user(&session).await,
    Err(ApiError::HttpStatus(503))
  ));
  assert!(matches!(
    client.get_user(&session).await,
    Err(ApiError::Unauthenticated)
  ));
  server.join().unwrap();
  assert!(matches!(
    client.get_user(&session).await,
    Err(ApiError::Network(_))
  ));
}
