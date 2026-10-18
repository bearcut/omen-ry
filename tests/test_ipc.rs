use omen_ry::engine::Engine;
use omen_ry::hal::mock::MockBackend;
use omen_ry::ipc::protocol::{IpcRequest, IpcResponse};

#[test]
fn test_ipc_serialization_roundtrip() {
    let req = IpcRequest::SetModeWave {
        palette: Some("matrix".to_string()),
        speed: Some(1.2),
        reverse: Some(true),
    };

    let serialized = serde_json::to_string(&req).expect("Failed to serialize request");
    let deserialized: IpcRequest = serde_json::from_str(&serialized).expect("Failed to deserialize");

    match deserialized {
        IpcRequest::SetModeWave { palette, speed, reverse } => {
            assert_eq!(palette.as_deref(), Some("matrix"));
            assert_eq!(speed, Some(1.2));
            assert_eq!(reverse, Some(true));
        }
        _ => panic!("Deserialized wrong request variant"),
    }
}

#[test]
fn test_engine_ipc_handling() {
    let backend = Box::new(MockBackend::new(false));
    let mut engine = Engine::new(backend, 80, 30);

    // Initial state
    assert!(engine.is_animating()); // wave is active

    // Set static colors
    let resp = engine.handle_ipc(IpcRequest::SetColorAll {
        color: "00FF00".to_string(),
    });
    match resp {
        IpcResponse::Ok { message } => assert!(message.contains("#00FF00")),
        _ => panic!("Expected Ok response"),
    }

    // Render static frame
    let _ = engine.render_frame(std::time::Duration::from_millis(0));

    // After static render, engine should be quiescent (not animating)
    assert!(!engine.is_animating());

    // Switch back to wave
    let _ = engine.handle_ipc(IpcRequest::SetModeWave {
        palette: Some("cyberpunk".to_string()),
        speed: Some(0.5),
        reverse: None,
    });
    assert!(engine.is_animating());

    // Test brightness command
    let _ = engine.handle_ipc(IpcRequest::SetBrightness { brightness: 50 });
    let status_resp = engine.handle_ipc(IpcRequest::GetStatus);
    match status_resp {
        IpcResponse::Status(s) => {
            assert_eq!(s.brightness, 50);
            assert_eq!(s.mode, "wave");
        }
        _ => panic!("Expected Status response"),
    }
}
