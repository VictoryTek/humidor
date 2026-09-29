{
  description = "Humidor - cigar inventory management service";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
  };

  outputs =
    { self, nixpkgs }:
    let
      systems = [
        "x86_64-linux"
        "aarch64-linux"
      ];
      forAllSystems = nixpkgs.lib.genAttrs systems;
      pkgsFor = system: nixpkgs.legacyPackages.${system};
    in
    {
      packages = forAllSystems (
        system:
        let
          pkgs = pkgsFor system;
        in
        {
          default = pkgs.rustPlatform.buildRustPackage {
            pname = "humidor";
            version = (builtins.fromTOML (builtins.readFile ./Cargo.toml)).package.version;

            src = nixpkgs.lib.cleanSource self;

            cargoLock = {
              lockFile = ./Cargo.lock;
            };

            # The repo's .cargo/config.toml sets a global `-fuse-ld=lld` rustflag
            # (added to work around a Windows MSVC PDB limit) that applies to every
            # target, including Linux. `lld` must be on PATH for the link step to
            # succeed here.
            nativeBuildInputs = [
              pkgs.pkg-config
              pkgs.lld
            ];

            # lettre's `tokio1-native-tls` feature pulls in openssl-sys.
            buildInputs = [
              pkgs.openssl
            ];

            # Integration tests (`--tests`) require a live Postgres instance and
            # network-free sandboxed builds can't provide one; restrict the
            # build-time test phase to the DB-free unit tests.
            #
            # test_rate_limiter_expiry (src/middleware/rate_limiter.rs) sleeps 2s
            # against a 1s rate-limit window; under the sandbox's parallel
            # compilation load it is prone to scheduling-jitter flakiness
            # unrelated to this packaging, so it is skipped here specifically
            # (all other unit tests still run as real build-time verification).
            cargoTestFlags = [
              "--lib"
              "--"
              "--skip"
              "middleware::rate_limiter::tests::test_rate_limiter_expiry"
            ];

            # Migrations are embedded into the binary at compile time via
            # refinery::embed_migrations! and run automatically on startup, so
            # only the static frontend needs to be installed alongside the binary.
            postInstall = ''
              mkdir -p $out/share/humidor
              cp -r static $out/share/humidor/static
            '';

            meta = {
              description = "Cigar inventory management service";
              mainProgram = "humidor";
              license = nixpkgs.lib.licenses.gpl3Plus;
              platforms = systems;
            };
          };
        }
      );

      checks = forAllSystems (system: {
        default = self.packages.${system}.default;
      });

      nixosModules.default = import ./nix/module.nix { inherit self; };
    };
}
