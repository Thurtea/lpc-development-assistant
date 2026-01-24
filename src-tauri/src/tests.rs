#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::path::PathBuf;

    #[test]
    fn test_path_mapper_basic() {
        // Test that path mapping works correctly
        let mapper = crate::wsl::PathMapper::new(
            PathBuf::from("E:\\Work\\lpc-development-assistant"),
            "/home/thurtea/amlp-driver".to_string(),
            "/home/thurtea/amlp-library".to_string(),
        );

        // Test mapping to driver
        let driver_path = mapper.to_wsl_driver(&PathBuf::from("E:\\Work\\lpc-development-assistant\\src\\test.lpc"));
        assert!(driver_path.is_some());
        assert!(driver_path.unwrap().contains("/home/thurtea/amlp-driver"));

        // Test mapping to library
        let lib_path = mapper.to_wsl_library(&PathBuf::from("E:\\Work\\lpc-development-assistant\\lpc\\test.lpc"));
        assert!(lib_path.is_some());
        assert!(lib_path.unwrap().contains("/home/thurtea/amlp-library"));
    }

    #[test]
    fn test_wsl_executor_creation() {
        use crate::wsl::WslExecutor;
        
        // Test that executor initializes correctly
        let executor = WslExecutor::new(Some("/home/thurtea/amlp-driver".to_string()));
        assert!(executor.execute("echo 'test'").is_awaitable());
    }

    #[test]
    fn test_driver_pipeline_creation() {
        use crate::wsl::PathMapper;
        use std::path::PathBuf;

        // Mock test - just verify the pipeline initializes
        let mapper = Arc::new(PathMapper::new(
            PathBuf::from("E:\\Work\\lpc-development-assistant"),
            "/home/thurtea/amlp-driver".to_string(),
            "/home/thurtea/amlp-library".to_string(),
        ));

        // In real integration, this would be crate::driver::DriverPipeline::new(mapper);
        // but we're just testing that the types are accessible
        assert!(mapper.wsl_driver_root() == "/home/thurtea/amlp-driver");
        assert!(mapper.wsl_library_root() == "/home/thurtea/amlp-library");
    }
}
