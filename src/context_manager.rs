use std::fs;
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use walkdir::WalkDir;
use anyhow::{Result, Context as AnyhowContext};

#[derive(Clone, Debug)]
pub struct ReferenceDocument {
    pub path: PathBuf,
    pub filename: String,
    pub content: String,
    pub doc_type: ReferenceType,
    pub size_bytes: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ReferenceType {
    Efuns,
    DriverSource,
    MudlibSource,
    Documentation,
    Unknown,
}

pub struct ContextManager {
    mud_references_path: PathBuf,
    templates_path: PathBuf,
    extracted_path: PathBuf,
    reference_cache: HashMap<String, ReferenceDocument>,
    cache_loaded: bool,
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
            reference_cache: HashMap::new(),
            cache_loaded: false,
        }
    }

    pub fn load_reference_cache(&mut self) -> Result<usize> {
        if self.cache_loaded {
            return Ok(self.reference_cache.len());
        }

        let mut count = 0;
        let relevant_extensions = ["txt", "md", "c", "h", "lpc", "json", "jsonl"];

        for entry in WalkDir::new(&self.mud_references_path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            let path = entry.path();
            
            let ext = match path.extension() {
                Some(e) => e.to_str().unwrap_or(""),
                None => continue,
            };

            if !relevant_extensions.contains(&ext) {
                continue;
            }

            match fs::read_to_string(path) {
                Ok(content) => {
                    let filename = path.file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("unknown")
                        .to_string();

                    let doc_type = Self::classify_reference_type(&filename, &content);
                    let size_bytes = content.len();

                    let cache_key = format!("{}/{}", 
                        path.parent()
                            .and_then(|p| p.file_name())
                            .and_then(|n| n.to_str())
                            .unwrap_or("root"),
                        filename
                    );

                    self.reference_cache.insert(cache_key, ReferenceDocument {
                        path: path.to_path_buf(),
                        filename,
                        content,
                        doc_type,
                        size_bytes,
                    });

                    count += 1;
                }
                Err(e) => {
                    eprintln!("Warning: could not read file {:?}: {}", path, e);
                }
            }
        }

        self.cache_loaded = true;
        Ok(count)
    }

    fn classify_reference_type(filename: &str, _content: &str) -> ReferenceType {
        let lower = filename.to_lowercase();
        
        if lower.contains("efun") {
            ReferenceType::Efuns
        } else if lower.contains("driver") || lower.contains("dgd") || lower.contains("fluffos") || lower.contains("ldmud") {
            ReferenceType::DriverSource
        } else if lower.contains("mudlib") || lower.contains("lib") {
            ReferenceType::MudlibSource
        } else if lower.ends_with(".md") || lower.ends_with(".txt") {
            ReferenceType::Documentation
        } else {
            ReferenceType::Unknown
        }
    }

    pub fn get_references_by_type(&self, doc_type: ReferenceType) -> Vec<ReferenceDocument> {
        self.reference_cache
            .values()
            .filter(|doc| doc.doc_type == doc_type)
            .cloned()
            .collect()
    }

    pub fn search_references(&self, keyword: &str) -> Vec<ReferenceDocument> {
        let keyword_lower = keyword.to_lowercase();
        let mut results = Vec::new();

        for doc in self.reference_cache.values() {
            if doc.content.to_lowercase().contains(&keyword_lower) ||
               doc.filename.to_lowercase().contains(&keyword_lower) {
                results.push(doc.clone());
            }
        }

        results
    }

    pub fn get_reference_snippet(&self, filename: &str, search_term: &str, context_lines: usize) -> Option<String> {
        for doc in self.reference_cache.values() {
            if doc.filename == filename || doc.filename.ends_with(filename) {
                let search_lower = search_term.to_lowercase();
                let content_lower = doc.content.to_lowercase();

                if let Some(pos) = content_lower.find(&search_lower) {
                    let lines: Vec<&str> = doc.content.lines().collect();
                    let byte_count: Vec<usize> = doc.content.lines()
                        .scan(0, |acc, line| { let len = *acc; *acc += line.len() + 1; Some(len) })
                        .collect();

                    let mut current_byte = 0;
                    let mut line_idx = 0;
                    for (i, &byte_pos) in byte_count.iter().enumerate() {
                        if byte_pos <= pos {
                            line_idx = i;
                            current_byte = byte_pos;
                        } else {
                            break;
                        }
                    }

                    let start_line = if line_idx > context_lines { line_idx - context_lines } else { 0 };
                    let end_line = std::cmp::min(line_idx + context_lines + 1, lines.len());

                    return Some(lines[start_line..end_line].join("\n"));
                }
            }
        }
        None
    }

    pub fn cache_stats(&self) -> (usize, usize) {
        let count = self.reference_cache.len();
        let size: usize = self.reference_cache.values().map(|d| d.size_bytes).sum();
        (count, size)
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
            .with_context(|| format!("Failed to load driver context from {:?}", template_path))
    }

    pub fn load_mudlib_context(&self) -> Result<String> {
        let template_path = self.templates_path.join("mudlib_context.txt");
        fs::read_to_string(&template_path)
            .with_context(|| format!("Failed to load mudlib context"))
    }

    pub fn load_efuns_context(&self) -> Result<String> {
        let template_path = self.templates_path.join("efuns_context.txt");
        fs::read_to_string(&template_path)
            .with_context(|| format!("Failed to load efuns context"))
    }

    pub fn load_reference_sources_context(&self) -> Result<String> {
        let template_path = self.templates_path.join("reference_sources.txt");
        fs::read_to_string(&template_path)
            .with_context(|| format!("Failed to load reference sources context"))
    }

    pub fn load_simul_efun_context(&self) -> Result<String> {
        let template_path = self.templates_path.join("simul_efun_summary.txt");
        fs::read_to_string(&template_path)
            .with_context(|| format!("Failed to load simul_efun summary context"))
    }

    pub fn load_master_api_context(&self) -> Result<String> {
        let template_path = self.templates_path.join("master_api.txt");
        fs::read_to_string(&template_path)
            .with_context(|| format!("Failed to load master api context"))
    }

    pub fn load_socket_api_context(&self) -> Result<String> {
        let template_path = self.templates_path.join("socket_api.txt");
        fs::read_to_string(&template_path)
            .with_context(|| format!("Failed to load socket api context"))
    }

    pub fn load_comm_context(&self) -> Result<String> {
        let template_path = self.templates_path.join("comm_summary.txt");
        fs::read_to_string(&template_path)
            .with_context(|| format!("Failed to load comm summary context"))
    }

    pub fn load_backend_context(&self) -> Result<String> {
        let template_path = self.templates_path.join("backend_loop.txt");
        fs::read_to_string(&template_path)
            .with_context(|| format!("Failed to load backend loop context"))
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
        self.create_template_if_missing("driver_codegen.txt", include_str!("../templates/driver_codegen.txt"))?;
        self.create_template_if_missing("object_system.txt", include_str!("../templates/object_system.txt"))?;

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
