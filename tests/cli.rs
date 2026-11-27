//! The command line end to end: the real binary against a throwaway library and a throwaway home.
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

struct Sandbox {
    root: PathBuf,
    lib: PathBuf,
}

impl Sandbox {
    fn new(tag: &str) -> Sandbox {
        let root = std::env::temp_dir().join(format!("skills-cli-{tag}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        let lib = root.join("lib");
        fs::create_dir_all(&lib).unwrap();
        fs::create_dir_all(root.join("home")).unwrap();
        let sb = Sandbox { root, lib };
        sb.skill("pdf-tools", "Work with PDF files: read, merge, split. Use when a task involves a PDF.");
        sb.skill("docker-help", "Build and debug containers. Use when a task involves Docker.");
        fs::create_dir_all(sb.lib.join("pdf-tools/references/guide")).unwrap();
        fs::write(sb.lib.join("pdf-tools/references/guide/merging.md"), "# Merging\nUse qpdf --empty --pages a.pdf b.pdf -- out.pdf\n").unwrap();
        fs::write(sb.lib.join("pdf-tools/references/overview.md"), "# Overview\nPDF basics and the exclusive-zone of nothing.\n").unwrap();
        fs::create_dir_all(sb.lib.join("pdf-tools/scripts")).unwrap();
        fs::write(sb.lib.join("pdf-tools/scripts/hello.sh"), "#!/bin/sh\necho hello from $1\n").unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(sb.lib.join("pdf-tools/scripts/hello.sh"), fs::Permissions::from_mode(0o755)).unwrap();
        }
        sb
    }

    fn skill(&self, name: &str, description: &str) {
        fs::create_dir_all(self.lib.join(name)).unwrap();
        fs::write(self.lib.join(name).join("SKILL.md"), format!("---\nname: {name}\ndescription: {description}\nlicense: MIT\n---\n\n# {name}\n\nInstructions for {name}. See [the guide](references/overview.md).\n")).unwrap();
    }

    fn run(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_skills"))
            .args(args)
            .env("HOME", self.root.join("home"))
            .env("XDG_CONFIG_HOME", self.root.join("home/.config"))
            .env("SKILLS_PATH", &self.lib)
            .env("NO_COLOR", "1")
            .current_dir(&self.root)
            .output()
            .unwrap()
    }
}

fn out(o: &Output) -> String { String::from_utf8_lossy(&o.stdout).into_owned() }
fn err(o: &Output) -> String { String::from_utf8_lossy(&o.stderr).into_owned() }

#[test]
fn version_and_skill_guide() {
    let sb = Sandbox::new("version");
    let v = sb.run(&["--version"]);
    assert!(out(&v).contains(env!("CARGO_PKG_VERSION")));
    let g = out(&sb.run(&["skill", "--raw"]));
    assert!(g.contains("skills install") && g.contains("skills validate"), "the built-in guide covers the commands");
    assert!(!g.to_lowercase().contains("cloudwerx"));
}

#[test]
fn list_search_and_load() {
    let sb = Sandbox::new("list");
    let list = out(&sb.run(&["list", "--short"]));
    assert_eq!(list.lines().filter(|l| !l.trim().is_empty()).count(), 2, "{list}");
    assert!(list.contains("pdf-tools") && list.contains("docker-help"));
    fs::create_dir_all(sb.lib.join(".system")).unwrap();
    fs::create_dir_all(sb.lib.join("not-a-skill")).unwrap();
    let again = out(&sb.run(&["list", "--short"]));
    assert!(!again.contains(".system") && !again.contains("not-a-skill"), "hidden folders and folders without a SKILL.md aren't skills: {again}");
    assert!(out(&sb.run(&["list", "--json"])).trim_start().starts_with('['));
    let s = out(&sb.run(&["search", "containers"]));
    assert!(s.contains("docker-help") && !s.contains("pdf-tools"), "{s}");
    let l = out(&sb.run(&["load", "pdf-tools", "--raw"]));
    assert!(l.contains("Instructions for pdf-tools"));
    let both = out(&sb.run(&["load", "pdf-tools", "docker-help"]));
    assert!(both.contains("Instructions for pdf-tools") && both.contains("Instructions for docker-help"));
    assert!(!sb.run(&["load", "nope"]).status.success());
}

#[test]
fn references_are_listed_searched_and_fetched_by_fuzzy_name() {
    let sb = Sandbox::new("ref");
    assert!(out(&sb.run(&["ref", "list", "pdf-tools"])).contains("merging.md"));
    assert!(out(&sb.run(&["ref", "search", "pdf-tools", "qpdf"])).contains("merging.md"));
    assert!(out(&sb.run(&["ref", "get", "pdf-tools", "merging"])).contains("qpdf --empty"));
    assert!(out(&sb.run(&["ref", "pdf-tools", "overview"])).contains("PDF basics"), "skill-first form");
    assert!(out(&sb.run(&["ref", "all", "pdf-tools"])).contains("Merging"));
    assert!(!sb.run(&["ref", "get", "pdf-tools", "does-not-exist"]).status.success());
}

#[test]
fn scripts_run_and_which_finds_paths() {
    let sb = Sandbox::new("run");
    assert!(out(&sb.run(&["run", "pdf-tools", "hello.sh", "world"])).contains("hello from world"));
    assert!(!sb.run(&["run", "pdf-tools", "missing.sh"]).status.success());
    let p = out(&sb.run(&["which", "pdf-tools"]));
    assert!(Path::new(p.trim()).join("SKILL.md").is_file(), "{p}");
    assert!(out(&sb.run(&["which", "pdf-tools", "--script", "hello.sh"])).trim().ends_with("hello.sh"));
}

#[test]
fn validate_passes_a_good_library_and_catches_a_broken_link() {
    let sb = Sandbox::new("validate");
    fs::create_dir_all(sb.lib.join("docker-help/references")).unwrap();
    fs::write(sb.lib.join("docker-help/references/overview.md"), "x").unwrap();
    assert!(sb.run(&["validate"]).status.success(), "{}", out(&sb.run(&["validate"])));
    fs::write(sb.lib.join("docker-help/SKILL.md"), "---\nname: docker-help\ndescription: Docker. Use when asked.\n---\n\nSee [gone](references/gone.md).\n").unwrap();
    let v = sb.run(&["validate", "docker-help"]);
    assert!(!v.status.success() || out(&v).to_lowercase().contains("gone"), "a link to a missing file is reported: {}", out(&v));
}

#[test]
fn new_install_and_remove_round_trip() {
    let sb = Sandbox::new("manage");
    let made = sb.run(&["new", "my-skill", "--to", sb.lib.to_str().unwrap(), "--description", "Does my thing. Use when asked."]);
    assert!(made.status.success(), "{}", err(&made));
    assert!(out(&sb.run(&["list", "--short"])).contains("my-skill"));
    assert!(sb.run(&["validate", "my-skill"]).status.success());
    assert!(!sb.run(&["new", "Bad Name", "--to", sb.lib.to_str().unwrap()]).status.success());

    let src = sb.root.join("src");
    fs::create_dir_all(src.join("shared-skill")).unwrap();
    fs::write(src.join("shared-skill/SKILL.md"), "---\nname: shared-skill\ndescription: Shared. Use when asked.\n---\n# shared\n").unwrap();
    let dest = sb.root.join("dest");
    let inst = sb.run(&["install", src.to_str().unwrap(), "--to", dest.to_str().unwrap()]);
    assert!(inst.status.success(), "{}", err(&inst));
    assert!(dest.join("shared-skill/SKILL.md").is_file());

    let dry = sb.run(&["remove", "my-skill"]);
    assert!(dry.status.success() && sb.lib.join("my-skill").exists(), "no --yes, nothing removed");
    assert!(sb.run(&["remove", "my-skill", "--yes"]).status.success());
    assert!(!sb.lib.join("my-skill").exists());
}

#[test]
fn config_and_doctor_never_touch_the_real_home() {
    let sb = Sandbox::new("config");
    let shown = out(&sb.run(&["config", "show"]));
    assert!(shown.contains(sb.lib.to_str().unwrap()), "{shown}");
    assert!(!shown.to_lowercase().contains("cloudwerx"));
    let path = out(&sb.run(&["config", "path"]));
    assert!(path.contains("skills-cli/config.toml"), "{path}");
    let d = sb.run(&["doctor"]);
    assert!(d.status.success(), "{}{}", out(&d), err(&d));
}

#[test]
fn pin_links_into_an_agent_folder_and_unpin_removes_only_the_link() {
    let sb = Sandbox::new("pin");
    let claude = sb.root.join("home/.claude/skills");
    fs::create_dir_all(&claude).unwrap();
    let pinned = sb.run(&["pin", "pdf-tools", "--agents", "claude"]);
    assert!(pinned.status.success(), "{}{}", out(&pinned), err(&pinned));
    assert!(claude.join("pdf-tools").is_symlink());
    assert!(out(&sb.run(&["pinned"])).contains("pdf-tools"));
    assert!(sb.run(&["unpin", "pdf-tools", "--agents", "claude"]).status.success());
    assert!(!claude.join("pdf-tools").exists());
    assert!(sb.lib.join("pdf-tools/SKILL.md").is_file(), "the library is untouched");
}

#[test]
fn completions_print_for_each_shell() {
    let sb = Sandbox::new("comp");
    for shell in ["fish", "bash", "zsh"] {
        let o = sb.run(&["completions", shell]);
        assert!(o.status.success() && out(&o).contains("skills"), "{shell}");
    }
}

#[test]
fn odd_input_is_an_error_never_a_panic() {
    let sb = Sandbox::new("odd");
    for args in [&["load"][..], &["ref"], &["ref", "get"], &["run", "pdf-tools"], &["search"], &["install"], &["new"], &["remove"], &["which", "nope"], &["files", "nope"], &["info", "nope"], &["pin"], &["bogus"], &["load", "../../etc/passwd"], &["ref", "get", "pdf-tools", "../../../../etc/passwd"], &["run", "pdf-tools", "../../../bin/sh"]] {
        let o = sb.run(args);
        assert!(!err(&o).contains("panicked"), "{args:?}: {}", err(&o));
    }
    let traversal = sb.run(&["ref", "get", "pdf-tools", "../../../../etc/passwd"]);
    assert!(!out(&traversal).contains("root:"), "a reference path can't leave the skill");
    let script = sb.run(&["run", "pdf-tools", "../../../bin/sh"]);
    assert!(!script.status.success(), "a script path can't leave the skill");
}
