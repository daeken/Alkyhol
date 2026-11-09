//! Explicit, separately built open compiler. Input/output stays in the caller's
//! private working directory; no shader data is written to a global temp cache.
use crate::{Result, legacy::Stage};
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::atomic::{AtomicU64, Ordering},
    time::{Duration, Instant},
};
static SERIAL: AtomicU64 = AtomicU64::new(0);
#[derive(Clone, Debug)]
pub struct Compiler {
    executable: PathBuf,
    directory: PathBuf,
}
#[derive(Debug)]
pub struct Input {
    pub semantic: String,
    pub index: u32,
    pub register: u32,
}
#[derive(Debug)]
pub struct Compiled {
    pub spirv: Vec<u8>,
    pub inputs: Vec<Input>,
}
/// Decode token extents, never search operand/comment payloads for END. SM1 has
/// implicit operand counts and remains explicitly unsupported by this reader.
pub fn bytecode(read: impl Fn(usize) -> Result<u32>, stage: Stage) -> Result<Vec<u8>> {
    let header = read(0)?;
    let tag = match stage {
        Stage::Vertex => 0xfffe,
        Stage::Pixel => 0xffff,
    };
    if header >> 16 != tag
        || !match ((header >> 8) & 255, header & 255) {
            (2, minor) => minor <= if stage == Stage::Pixel { 2 } else { 1 },
            (3, 0) => true,
            _ => false,
        }
    {
        return Err("legacy bytecode reader supports SM2/2a/2b and SM3.0 only".into());
    }
    let mut words = vec![header];
    while words.len() < 65536 {
        let token = read(words.len())?;
        words.push(token);
        if token == 0x0000ffff {
            return Ok(words.into_iter().flat_map(u32::to_le_bytes).collect());
        }
        let count = if token & 65535 == 0xfffe {
            ((token >> 16) & 0x7fff) as usize
        } else {
            ((token >> 24) & 15) as usize
        };
        if words.len() + count >= 65536 {
            return Err("legacy bytecode token budget exceeded".into());
        }
        for _ in 0..count {
            words.push(read(words.len())?);
        }
    }
    Err("legacy bytecode missing bounded END".into())
}
fn bounded(path: &Path, limit: u64) -> Result<Vec<u8>> {
    let file = fs::File::open(path).map_err(|e| e.to_string())?;
    if file.metadata().map_err(|e| e.to_string())?.len() > limit {
        return Err("compiler output budget exceeded".into());
    }
    use std::io::Read;
    let mut data = Vec::new();
    file.take(limit + 1)
        .read_to_end(&mut data)
        .map_err(|e| e.to_string())?;
    if data.len() as u64 > limit {
        return Err("compiler output changed beyond budget".into());
    }
    Ok(data)
}
impl Compiler {
    pub fn new(executable: &Path, directory: &Path) -> Result<Self> {
        let executable = executable
            .canonicalize()
            .map_err(|e| format!("legacy compiler executable: {e}"))?;
        fs::create_dir_all(directory).map_err(|e| e.to_string())?;
        let directory = directory.canonicalize().map_err(|e| e.to_string())?;
        Ok(Self {
            executable,
            directory,
        })
    }
    pub fn from_environment() -> Result<Self> {
        let executable = std::env::var_os("ALKYHOL_D3D9_SHADER_COMPILER")
            .ok_or("set ALKYHOL_D3D9_SHADER_COMPILER to the open compiler adapter")?;
        let directory = std::env::var_os("ALKYHOL_D3D9_SHADER_DIRECTORY")
            .ok_or("set ALKYHOL_D3D9_SHADER_DIRECTORY to a private local shader directory")?;
        Self::new(Path::new(&executable), Path::new(&directory))
    }
    pub fn compile(
        &self,
        bytes: &[u8],
        stage: Stage,
        following_pixel: Option<&[u8]>,
    ) -> Result<Compiled> {
        let validate = |bytes: &[u8], stage| -> Result<()> {
            let parsed = bytecode(
                |n| {
                    bytes
                        .get(n * 4..n * 4 + 4)
                        .map(|v| u32::from_le_bytes(v.try_into().unwrap()))
                        .ok_or("truncated bytecode".into())
                },
                stage,
            )?;
            if parsed != bytes {
                return Err("trailing bytecode beyond END".into());
            }
            Ok(())
        };
        validate(bytes, stage)?;
        if let Some(pixel) = following_pixel {
            if stage != Stage::Vertex {
                return Err("only a vertex stage can link a following pixel stage".into());
            }
            validate(pixel, Stage::Pixel)?;
        }
        let directory = loop {
            let path = self.directory.join(format!(
                "shader-{}-{}",
                std::process::id(),
                SERIAL.fetch_add(1, Ordering::Relaxed)
            ));
            match fs::create_dir(&path) {
                Ok(()) => break path,
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(e) => return Err(e.to_string()),
            }
        };
        let input = directory.join("input.d3dbc");
        let output = directory.join("output.spv");
        let reflection = directory.join("reflection.json");
        fs::write(&input, bytes).map_err(|e| e.to_string())?;
        let mut command = Command::new(&self.executable);
        command.arg(&input).arg(&output).arg(&reflection);
        if let Some(pixel) = following_pixel {
            let path = directory.join("following.d3dbc");
            fs::write(&path, pixel).map_err(|e| e.to_string())?;
            command.arg(path);
        }
        let log = directory.join("compiler.log");
        let mut child = command
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(fs::File::create(&log).map_err(|e| e.to_string())?)
            .spawn()
            .map_err(|e| e.to_string())?;
        let start = Instant::now();
        let status = loop {
            if let Some(status) = child.try_wait().map_err(|e| e.to_string())? {
                break status;
            }
            if start.elapsed() > Duration::from_secs(30) {
                let _ = child.kill();
                let _ = child.wait();
                return Err(format!(
                    "legacy compilation timed out; diagnostics: {}",
                    directory.display()
                ));
            }
            std::thread::sleep(Duration::from_millis(5));
        };
        if !status.success() {
            return Err(format!(
                "legacy compilation failed ({status}); {}",
                String::from_utf8_lossy(&bounded(&log, 65536)?)
            ));
        }
        let spirv = bounded(&output, 16 * 1024 * 1024)?;
        let record: serde_json::Value =
            serde_json::from_slice(&bounded(&reflection, 65536)?).map_err(|e| e.to_string())?;
        let group = if stage == Stage::Vertex { 0 } else { 1 };
        if record["version"] != 1
            || record["stage"] != group
            || record["compiler"] != "vkd3d-shader 2.1"
        {
            return Err("unsupported compiler reflection version/stage".into());
        }
        let inputs = record["inputs"]
            .as_array()
            .ok_or("missing compiler inputs")?
            .iter()
            .map(|v| {
                let number = |field: &str| {
                    v[field]
                        .as_u64()
                        .and_then(|n| u32::try_from(n).ok())
                        .ok_or_else(|| format!("invalid signature {field}"))
                };
                Ok(Input {
                    semantic: v["name"].as_str().ok_or("missing semantic")?.to_string(),
                    index: number("index")?,
                    register: number("register")?,
                })
            })
            .collect::<Result<Vec<_>>>()?;
        // Validate the actual output before exposing a live COM shader.
        let module = crate::legacy::lower_spirv(&spirv, stage)?;
        crate::legacy::wgsl(&module)?;
        Ok(Compiled { spirv, inputs })
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn token_extents_preserve_end_like_payloads() {
        let words = [
            0xfffe0300, 0x0002fffe, 0xffff, 0x1234, 0x02000001, 0xffff, 0x800f0000, 0xffff,
        ];
        let actual = bytecode(
            |n| words.get(n).copied().ok_or("truncated".into()),
            Stage::Vertex,
        )
        .unwrap();
        assert_eq!(actual.len(), words.len() * 4);
        assert!(
            bytecode(
                |n| words.get(n).copied().ok_or("truncated".into()),
                Stage::Pixel
            )
            .is_err()
        );
        assert!(
            bytecode(
                |n| words[..6].get(n).copied().ok_or("truncated".into()),
                Stage::Vertex
            )
            .is_err()
        );
        assert!(bytecode(|_| Ok(0xfffe0101), Stage::Vertex).is_err());
    }
}
