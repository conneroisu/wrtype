{
  stdenv,
  nixdoc,
  self,
  mdbook,
  mdbook-cmdrun,
  mdbook-open-on-gh,
  git,
}:

stdenv.mkDerivation {
  pname = "wrtype-docs-html";
  version = "0.1.0";
  src = self;
  sourceRoot = "source/docs";
  
  nativeBuildInputs = [
    nixdoc
    mdbook
    mdbook-open-on-gh
    mdbook-cmdrun
    git
  ];

  dontConfigure = true;
  dontFixup = true;

  env.RUST_BACKTRACE = 1;

  buildPhase = ''
    runHook preBuild
    
    # Trick open-on-gh to find the git root
    chmod +w ../ && mkdir -p ../.git
    
    # Generate API documentation from Rust source
    echo "# API Documentation" > src/api.md
    echo "" >> src/api.md
    echo "This section contains auto-generated API documentation from the Rust source code." >> src/api.md
    echo "" >> src/api.md
    echo "\`\`\`rust" >> src/api.md
    echo "// Main module structure" >> src/api.md
    cat ../src/main.rs | head -50 >> src/api.md
    echo "\`\`\`" >> src/api.md
    
    # Build the documentation
    mdbook build
    
    runHook postBuild
  '';

  installPhase = ''
    runHook preInstall
    mv book $out
    runHook postInstall
  '';

  meta = {
    description = "Documentation for wrtype - xdotool type for Wayland";
    longDescription = ''
      Complete documentation for wrtype, including user guides, developer
      documentation, API reference, and examples.
    '';
  };
}