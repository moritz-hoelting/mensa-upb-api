{
  pkgs,
  lib,
  config,
  inputs,
  ...
}:
{
  packages = [ pkgs.git pkgs.sqlx-cli ];

  languages.rust.enable = true;

  env.DATABASE_URL = "postgres://mensa-upb:mensa-upb@${config.services.postgres.listen_addresses}:${toString config.services.postgres.port}/mensa-upb";

  services.postgres = {
    enable = true;
    initialDatabases = [
      {
        name = "mensa-upb";
        user = "mensa-upb";
        pass = "mensa-upb";
      }
    ];
    listen_addresses = "localhost";
  };

  git-hooks.hooks = {
    clippy.enable = true;

    sqlx-prepare = {
      enable = true;

      name = "Run cargo sqlx prepare";
      entry = "cargo sqlx prepare --workspace --no-dotenv";
      pass_filenames = false;
    };
  };

  # See full reference at https://devenv.sh/reference/options/
}
