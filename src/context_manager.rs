use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use anyhow::{Result, Context};

pub struct ContextManager {
    mud_references_path: PathBuf,
    templates_path: PathBuf,
    extracted_path: PathBuf,
}

impl ContextManager {
    pub fn new(workspace_root: PathBuf) -> Self {
        let mud_references_path = workspace_root.join("mud-references");
        let templates_path = workspace_root.join("lpc-dev-assistant").join("templates");
        let extracted_path = mud_references_path.join("extracted");

        Self {
            mud_references_path,
            templates_path,
            extracted_path,
        }
    }

    pub fn extract_archives(&self) -> Result<()> {
        fs::create_dir_all(&self.extracted_path)?;

        for entry in WalkDir::new(&self.mud_references_path)
            .max_depth(1)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            
            if let Some(ext) = path.extension() {
                match ext.to_str() {
                    Some("zip") => self.extract_zip(path)?,
                    Some("tgz") | Some("gz") => self.extract_tar_gz(path)?,
                    _ => {}
                }
            }
        }
        Ok(())
    }

    fn extract_zip(&self, path: &Path) -> Result<()> {
        let file = fs::File::open(path)?;
        let mut archive = zip::ZipArchive::new(file)?;

        let file_stem = path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown");
        
        let dest_dir = self.extracted_path.join(file_stem);
        fs::create_dir_all(&dest_dir)?;

        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let outpath = match file.enclosed_name() {
                Some(path) => dest_dir.join(path),
                None => continue,
            };

            if file.name().ends_with('/') {
                fs::create_dir_all(&outpath)?;
            } else {
                if let Some(p) = outpath.parent() {
                    fs::create_dir_all(p)?;
                }
                let mut outfile = fs::File::create(&outpath)?;
                std::io::copy(&mut file, &mut outfile)?;
            }
        }
        Ok(())
    }

    fn extract_tar_gz(&self, path: &Path) -> Result<()> {
        use flate2::read::GzDecoder;
        use tar::Archive;

        let file = fs::File::open(path)?;
        let decoder = GzDecoder::new(file);
        let mut archive = Archive::new(decoder);

        let file_stem = path.file_stem()
            .and_then(|s| s.to_str())
            .and_then(|s| s.strip_suffix(".tar"))
            .unwrap_or("unknown");
        
        let dest_dir = self.extracted_path.join(file_stem);
        fs::create_dir_all(&dest_dir)?;
        archive.unpack(&dest_dir)?;
        Ok(())
    }

    pub fn load_driver_context(&self) -> Result<String> {
        let template_path = self.templates_path.join("driver_context.txt");
        fs::read_to_string(&template_path)
            .context("Failed to load driver context")
    }

    pub fn load_mudlib_context(&self) -> Result<String> {
        let template_path = self.templates_path.join("mudlib_context.txt");
        fs::read_to_string(&template_path)
            .context("Failed to load mudlib context")
    }

    pub fn load_efuns_context(&self) -> Result<String> {
        let template_path = self.templates_path.join("efuns_context.txt");
        fs::read_to_string(&template_path)
            .context("Failed to load efuns context")
    }

    pub fn load_reference_sources_context(&self) -> Result<String> {
        let template_path = self.templates_path.join("reference_sources.txt");
        fs::read_to_string(&template_path)
            .context("Failed to load reference sources context")
    }

    pub fn load_simul_efun_context(&self) -> Result<String> {
        let template_path = self.templates_path.join("simul_efun_summary.txt");
        fs::read_to_string(&template_path)
            .context("Failed to load simul_efun summary context")
    }

    pub fn load_master_api_context(&self) -> Result<String> {
        let template_path = self.templates_path.join("master_api.txt");
        fs::read_to_string(&template_path)
            .context("Failed to load master api context")
    }

    pub fn load_socket_api_context(&self) -> Result<String> {
        let template_path = self.templates_path.join("socket_api.txt");
        fs::read_to_string(&template_path)
            .context("Failed to load socket api context")
    }

    pub fn load_comm_context(&self) -> Result<String> {
        let template_path = self.templates_path.join("comm_summary.txt");
        fs::read_to_string(&template_path)
            .context("Failed to load comm summary context")
    }

    pub fn load_backend_context(&self) -> Result<String> {
        let template_path = self.templates_path.join("backend_loop.txt");
        fs::read_to_string(&template_path)
            .context("Failed to load backend loop context")
    }

    // Generic loader by filename for small snippet runners
    pub fn load_template_by_filename(&self, filename: &str) -> Result<String> {
        let template_path = self.templates_path.join(filename);
        fs::read_to_string(&template_path)
            .with_context(|| format!("Failed to load template {}", filename))
    }

    pub fn search_code_examples(&self, keyword: &str) -> Vec<PathBuf> {
        let mut results = Vec::new();
        let keyword_lower = keyword.to_lowercase();

        if !self.extracted_path.exists() {
            return results;
        }

        for entry in WalkDir::new(&self.extracted_path)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            
            if let Some(ext) = path.extension() {
                let ext_str = ext.to_str().unwrap_or("");
                if !matches!(ext_str, "c" | "h" | "lpc" | "y" | "l") {
                    continue;
                }
            } else {
                continue;
            }

            if let Ok(content) = fs::read_to_string(path) {
                if content.to_lowercase().contains(&keyword_lower) {
                    results.push(path.to_path_buf());
                }
            }

            if results.len() >= 50 {
                break;
            }
        }
        results
    }

    pub fn ensure_templates_exist(&self) -> Result<()> {
        fs::create_dir_all(&self.templates_path)?;
        
        // Create default templates with embedded content
        self.create_template_if_missing("driver_context.txt", include_str!("../templates/driver_context.txt"))?;
        self.create_template_if_missing("mudlib_context.txt", include_str!("../templates/mudlib_context.txt"))?;
        self.create_template_if_missing("efuns_context.txt", include_str!("../templates/efuns_context.txt"))?;
        self.create_template_if_missing("reference_sources.txt", include_str!("../templates/reference_sources.txt"))?;
        self.create_template_if_missing("simul_efun_summary.txt", include_str!("../templates/simul_efun_summary.txt"))?;
        self.create_template_if_missing("master_api.txt", include_str!("../templates/master_api.txt"))?;
        self.create_template_if_missing("socket_api.txt", include_str!("../templates/socket_api.txt"))?;
        self.create_template_if_missing("comm_summary.txt", include_str!("../templates/comm_summary.txt"))?;
        self.create_template_if_missing("backend_loop.txt", include_str!("../templates/backend_loop.txt"))?;

        // Additional snippet templates
        self.create_template_if_missing("socket_accept_flow.txt", include_str!("../templates/socket_accept_flow.txt"))?;
        self.create_template_if_missing("socket_partial_write.txt", include_str!("../templates/socket_partial_write.txt"))?;
        self.create_template_if_missing("comm_telnet_sb.txt", include_str!("../templates/comm_telnet_sb.txt"))?;
        self.create_template_if_missing("backend_select_loop.txt", include_str!("../templates/backend_select_loop.txt"))?;

        Ok(())
    }

    fn create_template_if_missing(&self, filename: &str, content: &str) -> Result<()> {
        let path = self.templates_path.join(filename);
        if !path.exists() {
            fs::write(&path, content)?;
        }
        Ok(())
    }
}
