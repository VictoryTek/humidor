{ self }:
{
  config,
  lib,
  pkgs,
  ...
}:
let
  cfg = config.services.humidor;
  inherit (lib)
    mkEnableOption
    mkOption
    mkIf
    mkMerge
    types
    ;

  usingDefaultDataDir = cfg.dataDir == "/var/lib/humidor";

  baseEnvironment = mkMerge [
    { PORT = toString cfg.port; }
    (mkIf (cfg.database.url != null) { DATABASE_URL = cfg.database.url; })
    (mkIf (cfg.database.url == null) {
      POSTGRES_HOST = cfg.database.host;
      POSTGRES_PORT = toString cfg.database.port;
      POSTGRES_USER = cfg.database.user;
      POSTGRES_DB = cfg.database.name;
    })
    {
      # Lets the app read back a previously-generated secret from dataDir.
      # NOTE: the app's *auto-generate-and-persist* path is hardcoded to
      # /app/data/jwt_secret (a Docker-specific path) and does not honor this
      # variable for writing, only for reading a pre-existing file. For
      # reliable persistence across restarts, supply JWT_SECRET via
      # `environmentFile` instead.
      JWT_SECRET_FILE = "${cfg.dataDir}/jwt_secret";
    }
  ];
in
{
  options.services.humidor = {
    enable = mkEnableOption "Humidor cigar inventory management service";

    package = mkOption {
      type = types.package;
      default = self.packages.${pkgs.stdenv.hostPlatform.system}.default;
      defaultText = lib.literalExpression "humidor.packages.<system>.default";
      description = "The Humidor package to run.";
    };

    port = mkOption {
      type = types.port;
      default = 9898;
      description = "Port the Humidor HTTP server listens on.";
    };

    dataDir = mkOption {
      type = types.path;
      default = "/var/lib/humidor";
      description = ''
        Directory used as the service's working directory. Holds the
        `backups/` and `uploads/` subdirectories the app writes to, a
        symlink to the packaged static frontend, and (if auto-generated)
        the persisted JWT secret.
      '';
    };

    environmentFile = mkOption {
      type = types.nullOr types.path;
      default = null;
      description = ''
        Path to an EnvironmentFile (systemd `EnvironmentFile=` syntax,
        `KEY=VALUE` per line) for secrets that should not live in the Nix
        store, such as `JWT_SECRET`, `POSTGRES_PASSWORD`, or SMTP
        credentials. Strongly recommended: set `JWT_SECRET` (32+ characters)
        here for production use, since the app's own auto-generated-secret
        persistence path is Docker-specific and unreliable under this
        module (see `dataDir` / `JWT_SECRET_FILE` note).
      '';
    };

    openFirewall = mkOption {
      type = types.bool;
      default = true;
      description = "Whether to open `port` in the firewall.";
    };

    database = {
      url = mkOption {
        type = types.nullOr types.str;
        default = null;
        description = ''
          Full `DATABASE_URL` connection string. Takes precedence over the
          discrete `database.*` options below, matching the app's own
          resolution order. Prefer setting this (or `POSTGRES_PASSWORD`)
          via `environmentFile` rather than here, to avoid storing
          credentials in the Nix store.
        '';
      };

      host = mkOption {
        type = types.str;
        default = "localhost";
        description = "Postgres host (used when `database.url` is not set).";
      };

      port = mkOption {
        type = types.port;
        default = 5432;
        description = "Postgres port (used when `database.url` is not set).";
      };

      user = mkOption {
        type = types.str;
        default = "humidor_user";
        description = "Postgres role name (used when `database.url` is not set).";
      };

      name = mkOption {
        type = types.str;
        default = "humidor_db";
        description = "Postgres database name (used when `database.url` is not set).";
      };

      createLocally = mkOption {
        type = types.bool;
        default = false;
        description = ''
          Provision a local `services.postgresql` instance with
          `ensureDatabases`/`ensureUsers` for `database.name`/`database.user`,
          and permit loopback-only, password-less (`trust`) connections for
          that user/database. Intended for single-host/local-only
          convenience (comparable to the Docker Compose deployment's own
          default credentials), not for multi-host production use. Set to
          `false` (the default) to use an externally-provided Postgres
          instance instead — set `database.host`/`database.port` (and
          `POSTGRES_PASSWORD` via `environmentFile`) or `database.url`.
        '';
      };
    };
  };

  config = mkIf cfg.enable (mkMerge [
    {
      networking.firewall.allowedTCPPorts = mkIf cfg.openFirewall [ cfg.port ];

      systemd.services.humidor = {
        description = "Humidor cigar inventory management service";
        wantedBy = [ "multi-user.target" ];
        after = [ "network-online.target" ] ++ lib.optional cfg.database.createLocally "postgresql.service";
        requires = lib.optional cfg.database.createLocally "postgresql.service";
        wants = [ "network-online.target" ];

        environment = baseEnvironment;

        preStart = ''
          mkdir -p backups uploads
          ln -sfn ${cfg.package}/share/humidor/static static
        '';

        serviceConfig = mkMerge [
          {
            ExecStart = "${cfg.package}/bin/humidor";
            WorkingDirectory = cfg.dataDir;
            Restart = "on-failure";
            RestartSec = "5s";

            EnvironmentFile = mkIf (cfg.environmentFile != null) cfg.environmentFile;

            NoNewPrivileges = true;
            PrivateTmp = true;
            ProtectSystem = "strict";
            ProtectHome = true;
            ReadWritePaths = [ cfg.dataDir ];
          }
          (
            if usingDefaultDataDir then
              {
                DynamicUser = true;
                StateDirectory = "humidor";
              }
            else
              {
                User = "humidor";
                Group = "humidor";
              }
          )
        ];
      };
    }

    (mkIf (!usingDefaultDataDir) {
      users.users.humidor = {
        isSystemUser = true;
        group = "humidor";
        home = cfg.dataDir;
      };
      users.groups.humidor = { };

      systemd.tmpfiles.rules = [
        "d ${cfg.dataDir} 0750 humidor humidor -"
      ];
    })

    (mkIf cfg.database.createLocally {
      services.postgresql = {
        enable = true;
        ensureDatabases = [ cfg.database.name ];
        ensureUsers = [
          {
            name = cfg.database.user;
            ensureDBOwnership = true;
          }
        ];
        # mkBefore: pg_hba matches the first applicable line, and the
        # module's own default rules (e.g. "host all all 127.0.0.1/32
        # scram-sha-256") would otherwise shadow this more specific rule.
        authentication = lib.mkBefore ''
          host  ${cfg.database.name}  ${cfg.database.user}  127.0.0.1/32  trust
        '';
      };
    })
  ]);
}
