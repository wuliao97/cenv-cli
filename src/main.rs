// use std::env;
use std::os::unix::fs::OpenOptionsExt;
use std::path::Path;
use std::fs::{create_dir, create_dir_all, File, OpenOptions};
use std::io::Write;
use std::process::Command;

use clap::{Parser, ValueEnum};
use anyhow::{Result, bail};


fn to_green(snippet: &str) -> String {
    format!("\x1b[38;5;10m{}\x1b[m", snippet)
}

fn to_red(snippet: &str) -> String {
    format!("\x1b[38;5;160m{}\x1b[m", snippet)
}


#[derive(Parser, Debug)]
#[clap(
    name = "cenv",
    version = "0.1.0",
    author = "Ennui", 
)]
struct Opt {
    /// Name of the Project.
    project_name: String,
    /// Default: CMake | you can select the Type for Build.
    #[clap(value_enum, default_value_t = BuildTools::CMake)]
    build_type: BuildTools,
    /// Default: false | If you want to use C lang.  
    #[clap(short, long, default_value_t = false)]
    c: bool,
    /// Default: true  | If you want to use C++ lang.
    #[clap(short = 'x', long, default_value_t = true)]
    cpp: bool,
    /// Initialization git and add a .gitignore.
    #[clap(short = 'g', long, default_value_t = false)]
    git: bool,
    /// Add a readme.md file.
    #[clap(short = 'r', long, default_value_t = false)]
    readme: bool
}

impl Opt {
    fn check_path(&self) -> Result<()> {
        let path = Path::new(&self.project_name);
        if path.exists() {
            bail!("{} The `{}` is Already Exising!", to_red("✘"), path.display());
        }

        Ok(())
    }

    fn lang(&self) -> Language {
        if self.c {
            Language::C
        } else {
            Language::Cpp
        }
    }

    fn check_build_tools(&self) -> BuildTools {
        let lang = self.lang();
        let is_c = lang == Language::C;

        if self.build_type == BuildTools::Gcc && is_c {
            return BuildTools::Gcc;
        }

        if self.build_type == BuildTools::Clang && is_c {
            return BuildTools::Clang;
        }
        
        self.build_type.clone()
    }
}

#[derive(Debug, PartialEq)]
enum Language {
    C,
    Cpp,
}

impl ToString for Language {
    fn to_string(&self) -> String {
        let name = match &self {
            Language::C => "C",
            Language::Cpp => "C++",
        };
        name.to_string()
    }
}

#[derive(ValueEnum, Debug, Clone, PartialEq)]
enum BuildTools {
    Gcc,
    Gpp,
    #[clap(name = "cmake")]
    CMake,
    Clang,
    Clangpp,
}

impl ToString for BuildTools {
    fn to_string(&self) -> String {
        let name = match &self {
            BuildTools::Gcc => "gcc",
            BuildTools::Gpp => "g++",
            BuildTools::CMake => "CMake",
            BuildTools::Clang => "Clang",
            BuildTools::Clangpp => "Clang++",
        };
        name.to_string()
    }
}

impl BuildTools {
    fn generate_project<P: AsRef<Path>>(
        &self, path: P, p_name: &str, lang: &Language, git: bool, readme: bool
    ) -> Result<()> {
        Self::base_gen(&path, lang, git, readme).ok();

        match &self {
            BuildTools::Gcc => Self::gcc_gen(path, lang),
            BuildTools::Gpp => Self::gpp_gen(path, lang),
            BuildTools::CMake => Self::cmake_gen(path, p_name, lang),
            BuildTools::Clang => Self::clang_gen(path, lang),
            BuildTools::Clangpp => Self::clangpp_gen(path, lang),
        }
    }

    fn base_gen<P: AsRef<Path>>(path: P, lang: &Language, git: bool, readme: bool) -> Result<()> {
        let src_path = path.as_ref().join("src");
        create_dir_all(&src_path).unwrap();
        
        let (lib_name, ext) = if lang == &Language::C { ("stdio.h", ".c") } else { ("iostream", ".cpp") };
        let main_path = src_path.join(format!("main{}", ext));
        let buf = format!("#include <{}>\n\n\
            int main() {{\n\n\
            }}\n", lib_name);

        File::create(main_path).unwrap()
            .write_all(buf.as_bytes()).unwrap();

        if git {
            let result = Command::new("git")
                            .arg("init")
                            .current_dir(&path)
                            .output()
                            .unwrap();
            if !result.status.success() {
                bail!("Fatal error to Git initialization:\nis it might...\n\
                    \t1. Dosen't installed Git.\n\
                    \t2. Path dosen't currect");
            }
            // i didnt test this code 🙃
            let git_ign_path = path.as_ref().join(".gitignore"); 
            File::create(git_ign_path).unwrap().write_all("build".as_bytes()).unwrap();            
        };

        if readme {
            let readme_path = path.as_ref().join("readme.md"); 
            File::create(readme_path).unwrap();
        };

        Ok(())
    }

    fn gcc_gen<P: AsRef<Path>>(path: P, lang: &Language) -> Result<()> {
        let ext = if lang == &Language::C { ".c" } else { ".cpp" };
        let buf = format!("gcc -o main ./src/main{} && ./main", ext);
        let run_path = path.as_ref().join("run");

        let mut run = OpenOptions::new()
            .create(true)
            .write(true)
            .mode(0o744)
            .open(run_path)
            .unwrap();

        run.write_all(buf.as_bytes()).unwrap();

        Ok(())
    }

    fn gpp_gen<P: AsRef<Path>>(path: P, lang: &Language) -> Result<()> {
        let ext = if lang == &Language::C { ".c" } else { ".cpp" };
        let buf = format!("g++ -o main ./src/main{} && ./main", ext);
        let run_path = path.as_ref().join("run");

        let mut run = OpenOptions::new()
            .create(true)
            .write(true)
            .mode(0o744)
            .open(run_path)
            .unwrap();

        run.write_all(buf.as_bytes()).unwrap();

        Ok(())
    }

    fn cmake_gen<P: AsRef<Path>>(path: P, p_name: &str, lang: &Language) -> Result<()> {
        let build_path = path.as_ref().join("build");
        create_dir(build_path).unwrap();

        let mut sb = String::new();
        sb.push_str("cmake_minimum_required(VERSION 3.14)\n");
        
        let (lang, ext) = if lang == &Language::C { ("C", ".c") } else { ("CXX", ".cpp") };

        let project_line = format!("\nproject({} {})", p_name, lang);
        sb.push_str(&project_line);

        let src = format!("\n\nset(src\nsrc/main{}\n)\n\n", ext);
        sb.push_str(&src);

        let execute_line = format!("add_executable({} ${{src}})", p_name);
        sb.push_str(&execute_line);

        let cmake_path = path.as_ref().join("CMakeLists.txt");
        File::create(cmake_path).unwrap()
            .write_all(sb.as_bytes()).unwrap();

        Ok(())
    }

    fn clang_gen<P: AsRef<Path>>(path: P, lang: &Language) -> Result<()> {
        let ext = if lang == &Language::C { ".c" } else { ".cpp" };
        let buf = format!("clang -o main ./src/main{} && ./main", ext);

        let run_path = path.as_ref().join("run");
        let mut run = OpenOptions::new()
            .create(true)
            .write(true)
            .mode(0o744)
            .open(run_path)
            .unwrap();

        run.write_all(buf.as_bytes()).unwrap();

        Ok(())
    }

    fn clangpp_gen<P: AsRef<Path>>(path: P, lang: &Language) -> Result<()> {
        let ext = if lang == &Language::C { ".c" } else { ".cpp" };
        let buf = format!("clang++ -o main ./src/main{} && ./main", ext);
        let run_path = path.as_ref().join("run");

        let mut run = OpenOptions::new()
            .create(true)
            .write(true)
            .mode(0o744)
            .open(run_path)
            .unwrap();

        run.write_all(buf.as_bytes()).unwrap();

        Ok(())
    }
}



fn main() -> Result<()> {
    use std::process::exit;

    let arg = Opt::parse();

    if let Err(path_err) = arg.check_path() {
        eprintln!("{}", path_err);
        exit(1);
    }    

    let build_tools = arg.check_build_tools();
    let path = &arg.project_name;
    let project_name = &arg.project_name;
    let lang = arg.lang();


    if let Err(why) = build_tools.generate_project(path,&project_name, &lang, arg.git, arg.readme) {
        eprintln!("{}", why);
        exit(1);
    }

    println!(r"{} Successfully Generated!
    Project name: {}
    Language    : {}
    Build Type  : {}",
        to_green("✓"),
        arg.project_name,
        arg.lang().to_string(),
        build_tools.to_string()
    );

    Ok(())
}
