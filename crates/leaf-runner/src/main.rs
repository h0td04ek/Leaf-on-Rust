use std::env;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

const USAGE: &str = "\
leaf-runner — запуск Leaf server на jvm25-rust

Использование:
  leaf-runner [опции] [-- аргументы сервера...]

Опции:
  --jvm <path>      путь к бинарю jvm25-rust
                    (по умолчанию java-25-on-rust/target/release/jvm25-rust[.exe]
                     или env JVM25_RUST)
  --modules <path>  путь к lib/modules JDK 25
                    (по умолчанию <java.home>/lib/modules)
  --jar <path>      jar сервера Leaf
                    (по умолчанию новейший leaf-server/build/libs/leaf-*.jar)
  --java-home <dir> java.home для --modules
  -h, --help        эта справка

Всё после `--` передаётся серверу как есть (например: nogui --world foo).
";

struct Config {
    jvm: PathBuf,
    modules: Option<PathBuf>,
    jar: Option<PathBuf>,
    classpath: Option<String>,
    main_class: Option<String>,
    java_home: Option<PathBuf>,
    server_args: Vec<OsString>,
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("crates/leaf-runner -> workspace root")
        .to_path_buf()
}

fn default_jvm(root: &Path) -> PathBuf {
    let name = if cfg!(windows) {
        "jvm25-rust.exe"
    } else {
        "jvm25-rust"
    };
    root.join("java-25-on-rust")
        .join("target")
        .join("release")
        .join(name)
}

fn detect_java_home() -> Option<PathBuf> {
    if let Ok(home) = env::var("JAVA_HOME") {
        let p = PathBuf::from(home);
        if p.join("lib").join("modules").exists() {
            return Some(p);
        }
    }
    // Fallback: спросить у java
    let out = Command::new("java")
        .args(["-XshowSettings:properties", "-version"])
        .output()
        .ok()?;
    let text = String::from_utf8_lossy(&out.stderr);
    for line in text.lines() {
        if let Some(rest) = line.trim().strip_prefix("java.home =") {
            let p = PathBuf::from(rest.trim());
            if p.join("lib").join("modules").exists() {
                return Some(p);
            }
        }
    }
    None
}

fn default_jar(root: &Path) -> Option<PathBuf> {
    let libs = root.join("leaf-server").join("build").join("libs");
    let mut jars: Vec<PathBuf> = std::fs::read_dir(&libs)
        .ok()?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.starts_with("leaf-") && n.ends_with(".jar"))
        })
        .collect();
    jars.sort_by_key(|p| {
        std::fs::metadata(p)
            .and_then(|m| m.modified())
            .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
    });
    jars.pop()
}

fn parse_args() -> Result<Config, String> {
    let root = workspace_root();
    let mut cfg = Config {
        jvm: env::var_os("JVM25_RUST")
            .map(PathBuf::from)
            .unwrap_or_else(|| default_jvm(&root)),
        modules: None,
        jar: None,
        classpath: None,
        main_class: None,
        java_home: None,
        server_args: Vec::new(),
    };
    let mut args = env::args_os().skip(1).peekable();
    let mut passthrough = false;
    while let Some(arg) = args.next() {
        if passthrough {
            cfg.server_args.push(arg);
            continue;
        }
        match arg.to_str() {
            Some("--") => passthrough = true,
            Some("-h") | Some("--help") => {
                print!("{USAGE}");
                std::process::exit(0);
            }
            Some("--jvm") => {
                cfg.jvm = PathBuf::from(args.next().ok_or("--jvm: нет пути")?);
            }
            Some("--modules") => {
                cfg.modules = Some(PathBuf::from(args.next().ok_or("--modules: нет пути")?));
            }
            Some("--jar") => {
                cfg.jar = Some(PathBuf::from(args.next().ok_or("--jar: нет пути")?));
            }
            Some("--cp") | Some("--classpath") => {
                let v = args.next().ok_or("--cp: нет пути")?.to_string_lossy().into_owned();
                // @file — прочитать classpath из файла (например, вывод Gradle)
                cfg.classpath = Some(if let Some(path) = v.strip_prefix('@') {
                    std::fs::read_to_string(path)
                        .map_err(|e| format!("не удалось прочитать classpath из {path}: {e}"))?
                        .trim()
                        .to_owned()
                } else {
                    v
                });
            }
            Some("--main") => {
                cfg.main_class = Some(
                    args.next()
                        .ok_or("--main: нет класса")?
                        .to_string_lossy()
                        .into_owned(),
                );
            }
            Some("--java-home") => {
                cfg.java_home = Some(PathBuf::from(args.next().ok_or("--java-home: нет пути")?));
            }
            Some(other) if other.starts_with('-') => {
                return Err(format!("неизвестная опция: {other}"));
            }
            _ => {
                cfg.server_args.push(arg);
            }
        }
    }
    if cfg.jar.is_none() && cfg.main_class.is_none() {
        cfg.jar = Some(default_jar(&root).ok_or_else(|| {
            "не найден leaf-*.jar в leaf-server/build/libs — собери проект: ./gradlew :leaf-server:build"
                .to_string()
        })?);
    }
    if let Some(jar) = &cfg.jar {
        if !jar.exists() {
            return Err(format!("jar не найден: {}", jar.display()));
        }
    }
    if cfg.main_class.is_some() && cfg.classpath.is_none() {
        return Err("--main требует --cp <classpath>".to_string());
    }
    if cfg.modules.is_none() {
        let home = cfg.java_home.clone().or_else(detect_java_home).ok_or_else(|| {
            "не удалось найти java.home с lib/modules — укажи --modules или --java-home"
                .to_string()
        })?;
        let modules = home.join("lib").join("modules");
        if !modules.exists() {
            return Err(format!("modules image не найден: {}", modules.display()));
        }
        cfg.modules = Some(modules);
    }
    Ok(cfg)
}

fn main() -> ExitCode {
    let cfg = match parse_args() {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("leaf-runner: {e}\n\n{USAGE}");
            return ExitCode::from(2);
        }
    };
    if !cfg.jvm.exists() {
        eprintln!(
            "leaf-runner: jvm25-rust не найден: {}\nСобери JVM: cd java-25-on-rust && cargo build --release",
            cfg.jvm.display()
        );
        return ExitCode::from(2);
    }
    println!("[leaf-runner] jvm:     {}", cfg.jvm.display());
    println!("[leaf-runner] modules: {}", cfg.modules.as_ref().unwrap().display());
    match (&cfg.jar, &cfg.main_class) {
        (Some(jar), _) => println!("[leaf-runner] jar:     {}", jar.display()),
        (None, Some(main)) => {
            println!("[leaf-runner] main:    {main}");
            println!("[leaf-runner] cp:      {} entries", cfg.classpath.as_deref().unwrap_or("").split(';').count());
        }
        (None, None) => unreachable!(),
    }
    println!("[leaf-runner] args:    {:?}", cfg.server_args);

    // Paper/Leaf ждёт стандартные каталоги в рабочей директории;
    // plugins обязателен — без него DirectoryProviderSource падает с NoSuchFileException.
    for dir in ["plugins", "cache", "libraries", "versions", "logs", "world"] {
        let p = Path::new(dir);
        if let Err(e) = std::fs::create_dir_all(p) {
            eprintln!("leaf-runner: не удалось создать {dir}: {e}");
            return ExitCode::FAILURE;
        }
    }

    let mut cmd = Command::new(&cfg.jvm);
    cmd.arg("--modules").arg(cfg.modules.as_ref().unwrap());
    match (&cfg.jar, &cfg.classpath, &cfg.main_class) {
        (Some(jar), _, _) => {
            cmd.arg("-jar").arg(jar);
        }
        (None, Some(cp), Some(main)) => {
            cmd.arg("-cp").arg(cp).arg(main);
        }
        _ => unreachable!(),
    }
    let status = cmd.args(&cfg.server_args).status();
    match status {
        Ok(s) => match s.code() {
            Some(code) => ExitCode::from(code as u8),
            None => ExitCode::FAILURE,
        },
        Err(e) => {
            eprintln!("leaf-runner: не удалось запустить {}: {e}", cfg.jvm.display());
            ExitCode::FAILURE
        }
    }
}
