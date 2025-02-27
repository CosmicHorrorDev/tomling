#[test]
fn zbus() {
    use tomling::{parse, Value};

    let parsed_map = parse(CARGO_TOML).unwrap();

    // Too much to check for everything. Let's check some keys and values.
    let package = match parsed_map.get("package").unwrap() {
        Value::Table(package) => package,
        _ => panic!(),
    };
    assert_eq!(package.get("name").unwrap(), &Value::String("zbus"));
    assert_eq!(package.get("version").unwrap(), &Value::String("5.1.1"));
    assert_eq!(package.get("edition").unwrap(), &Value::String("2021"));

    // Let's check the dependencies, especially the complicated ones.
    let dependencies = match parsed_map.get("dependencies").unwrap() {
        Value::Table(dependencies) => dependencies,
        _ => panic!(),
    };

    // Serde
    let serde = match dependencies.get("serde").unwrap() {
        Value::Table(serde) => serde,
        _ => panic!(),
    };
    assert_eq!(serde.get("version").unwrap(), &Value::String("1.0.200"));
    assert_eq!(
        serde.get("features").unwrap(),
        &Value::Array([Value::String("derive")].into_iter().collect())
    );
    // Tokio
    let tokio = match dependencies.get("tokio").unwrap() {
        Value::Table(tokio) => tokio,
        _ => panic!(),
    };
    assert_eq!(tokio.get("version").unwrap(), &Value::String("1.37.0"));
    assert_eq!(tokio.get("optional").unwrap(), &Value::Boolean(true));
    assert_eq!(
        tokio.get("features").unwrap(),
        &Value::Array(
            [
                Value::String("rt"),
                Value::String("net"),
                Value::String("time"),
                Value::String("fs"),
                Value::String("io-util"),
                Value::String("process"),
                Value::String("sync"),
                Value::String("tracing"),
            ]
            .into_iter()
            .collect()
        )
    );

    // cfg-using dependencies
    let target = match parsed_map.get("target") {
        Some(Value::Table(target)) => target,
        _ => panic!(),
    };
    // Nix
    let nix = target
        .get("cfg(unix)")
        .and_then(|c| match c {
            Value::Table(c) => c.get("dependencies"),
            _ => None,
        })
        .and_then(|d| match d {
            Value::Table(d) => d.get("nix"),
            _ => None,
        })
        .and_then(|n| match n {
            Value::Table(n) => Some(n),
            _ => None,
        })
        .unwrap();
    assert_eq!(nix.get("version").unwrap(), &Value::String("0.29"));
    assert_eq!(nix.get("default-features").unwrap(), &Value::Boolean(false));
    assert_eq!(
        nix.get("features").unwrap(),
        &Value::Array(
            [
                Value::String("socket"),
                Value::String("uio"),
                Value::String("user"),
            ]
            .into_iter()
            .collect()
        )
    );
    // async-recursion
    let version = target
        .get("cfg(any(target_os = \"macos\", windows))")
        .and_then(|c| match c {
            Value::Table(c) => c.get("dependencies"),
            _ => None,
        })
        .and_then(|d| match d {
            Value::Table(d) => d.get("async-recursion"),
            _ => None,
        })
        .and_then(|a| match a {
            Value::String(a) => Some(a),
            _ => None,
        })
        .unwrap();
    assert_eq!(*version, "1.1.1");

    // Now array of tables
    let bench = match parsed_map.get("bench") {
        Some(Value::Array(bench)) => bench.get(0),
        _ => None,
    }
    .and_then(|b| match b {
        Value::Table(b) => Some(b),
        _ => None,
    })
    .unwrap();
    assert_eq!(bench.get("name").unwrap(), &Value::String("benchmarks"));
    assert_eq!(bench.get("harness").unwrap(), &Value::Boolean(false));

    // Finally, the examples
    let examples = match parsed_map.get("example") {
        Some(Value::Array(example)) => example,
        _ => panic!(),
    };
    let names = ["screen-brightness", "screen-brightness2"];
    let paths = [
        "examples/screen-brightness.rs",
        "examples/screen-brightness2.rs",
    ];
    for (i, example) in examples.iter().enumerate() {
        let example = match example {
            Value::Table(e) => e,
            _ => panic!(),
        };
        assert_eq!(example.get("name").unwrap(), &Value::String(names[i]));
        assert_eq!(example.get("path").unwrap(), &Value::String(paths[i]));
        assert_eq!(
            example.get("required-features").unwrap(),
            &Value::Array([Value::String("blocking-api")].into_iter().collect())
        );
    }
}

#[cfg(feature = "cargo-toml")]
#[test]
fn zbus_serde() {
    use tomling::cargo::{Dependency, LibraryType, Manifest, RustEdition};

    let manifest: Manifest = tomling::from_str(CARGO_TOML).unwrap();

    assert_eq!(manifest.package().name(), "zbus");
    assert_eq!(manifest.package().version(), "5.1.1");
    assert_eq!(manifest.package().edition().unwrap(), RustEdition::E2021);

    let serde = match manifest.dependencies().unwrap().by_name("serde").unwrap() {
        Dependency::Full(serde) => serde,
        _ => panic!(),
    };
    assert_eq!(serde.version(), "1.0.200");
    assert_eq!(serde.features(), Some(&["derive"][..]));

    let tokio = match manifest.dependencies().unwrap().by_name("tokio").unwrap() {
        Dependency::Full(tokio) => tokio,
        _ => panic!(),
    };
    assert_eq!(tokio.version(), "1.37.0");
    assert!(tokio.optional().unwrap());
    assert_eq!(
        tokio.features(),
        Some(&["rt", "net", "time", "fs", "io-util", "process", "sync", "tracing"][..])
    );

    // The library section.
    let lib = manifest.library().unwrap();
    assert!(!lib.bench().unwrap());
    assert_eq!(
        lib.library_type().unwrap(),
        &[LibraryType::Cdylib, LibraryType::Rlib]
    );

    // The benchmarks.
    let bench = manifest.benches().unwrap().first().unwrap();
    assert_eq!(bench.name(), "benchmarks");
    assert!(!bench.harness().unwrap());
}

const CARGO_TOML: &str = r#"
    [package]
    name = "zbus"
    version = "5.1.1"
    authors = ["Zeeshan Ali Khan <zeeshanak@gnome.org>"]
    edition = "2021"
    rust-version = "1.80"

    description = "API for D-Bus communication"
    repository = "https://github.com/dbus2/zbus/"
    keywords = ["D-Bus", "DBus", "IPC"]
    license = "MIT"
    categories = ["os::unix-apis"]
    readme = "README.md"

    [features]
    default = ["async-io", "blocking-api"]
    uuid = ["zvariant/uuid"]
    url = ["zvariant/url"]
    time = ["zvariant/time"]
    chrono = ["zvariant/chrono"]
    heapless = ["zvariant/heapless"]
    # Enables ser/de of `Option<T>` as an array of 0 or 1 elements.
    option-as-array = ["zvariant/option-as-array"]
    camino = ["zvariant/camino"]
    # Enables API that is only needed for bus implementations (enables `p2p`).
    bus-impl = ["p2p"]
    # Enables API that is only needed for peer-to-peer (p2p) connections.
    p2p = ["dep:rand"]
    async-io = [
        "dep:async-io",
        "async-executor",
        "async-task",
        "async-lock",
        "async-fs",
        # FIXME: We only currently only need this for unix but Cargo doesn't provide a way to enable
        # features for only specific target OS: https://github.com/rust-lang/cargo/issues/1197.
        "async-process",
        "blocking",
        "futures-util/io",
    ]
    tokio = ["dep:tokio"]
    vsock = ["dep:vsock", "dep:async-io"]
    tokio-vsock = ["dep:tokio-vsock", "tokio"]
    # Enable blocking API (default).
    blocking-api = ["zbus_macros/blocking-api"]
    # Enable `serde_bytes` feature of `zvariant`.
    serde_bytes = ["zvariant/serde_bytes"]

    [dependencies]
    zbus_macros = { path = "../zbus_macros", version = "=5.1.1" }
    zvariant = { path = "../zvariant", version = "5.0.0", default-features = false, features = [
        "enumflags2",
    ] }
    zbus_names = { path = "../zbus_names", version = "4.0" }
    serde = { version = "1.0.200", features = ["derive"] }
    serde_repr = "0.1.19"
    enumflags2 = { version = "0.7.9", features = ["serde"] }
    futures-core = "0.3.30"
    futures-util = { version = "0.3.30", default-features = false, features = [
        "std",
    ] }
    async-broadcast = "0.7.0"
    hex = "0.4.3"
    ordered-stream = "0.2"
    rand = { version = "0.8.5", optional = true }
    event-listener = "5.3.0"
    static_assertions = "1.1.0"
    async-trait = "0.1.80"
    xdg-home = "1.1.0"
    tracing = "0.1.40"
    winnow = "0.6"

    # Optional and target-specific dependencies.

    async-io = { version = "2.3.2", optional = true }
    async-lock = { version = "3.3.0", optional = true }
    async-executor = { version = "1.11.0", optional = true }
    blocking = { version = "1.6.0", optional = true }
    async-task = { version = "4.7.1", optional = true }
    async-fs = { version = "2.1.2", optional = true }
    async-process = { version = "2.2.2", optional = true }
    tokio = { version = "1.37.0", optional = true, features = [
        "rt",
        "net",
        "time",
        "fs",
        "io-util",
        # FIXME: We should only enable this feature for unix. See comment above regarding `async-process`
        # on why we can't.
        "process",
        "sync",
        "tracing",
    ] }
    vsock = { version = "0.5.0", optional = true }
    tokio-vsock = { version = "0.6", optional = true }

    [target.'cfg(windows)'.dependencies]
    windows-sys = { version = "0.59", features = [
    "Win32_Foundation",
    "Win32_Security_Authorization",
    "Win32_System_Memory",
    "Win32_Networking",
    "Win32_Networking_WinSock",
    "Win32_NetworkManagement",
    "Win32_NetworkManagement_IpHelper",
    "Win32_System_IO",
    "Win32_System_Threading",
    ] }
    uds_windows = "1.1.0"

    [target.'cfg(unix)'.dependencies]
    nix = { version = "0.29", default-features = false, features = [
        "socket",
        "uio",
        "user",
    ] }

    [target.'cfg(any(target_os = "macos", windows))'.dependencies]
    async-recursion = "1.1.1"

    [dev-dependencies]
    zbus_xml = { path = "../zbus_xml", version = "5.0.0" }
    doc-comment = "0.3.3"
    futures-util = "0.3.30" # activate default features
    ntest = "0.9.2"
    test-log = { version = "0.2.16", features = [
    "trace",
    ], default-features = false }
    tokio = { version = "1.37.0", features = [
        "macros",
        "rt-multi-thread",
        "fs",
        "io-util",
        "net",
        "sync",
        "time",
        "test-util",
    ] }
    tracing-subscriber = { version = "0.3.18", features = [
        "env-filter",
        "fmt",
        "ansi",
    ], default-features = false }
    tempfile = "3.10.1"
    criterion = "0.5.1"

    [package.metadata.docs.rs]
    all-features = true
    targets = ["x86_64-unknown-linux-gnu"]

    [lints]
    workspace = true

    [lib]
    bench = false
    # Note: zbus' Cargo.toml doesn't have a `crate-type` specified.
    crate-type = ["cdylib", "rlib"]

    [[bench]]
    name = "benchmarks"
    harness = false

    [[example]]
    name = "screen-brightness"
    path = "examples/screen-brightness.rs"
    required-features = ["blocking-api"]

    # No such example in zbus' Cargo.toml but we want a case of > 1 entry in an array of tables.
    [[example]]
    name = "screen-brightness2"
    path = "examples/screen-brightness2.rs"
    required-features = ["blocking-api"]
"#;
