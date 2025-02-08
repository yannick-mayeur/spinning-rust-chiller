{
  description = "HDD temperature-based fan control";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, rust-overlay }:
    let
      supportedSystems = [ "x86_64-linux" "aarch64-linux" ];
      forAllSystems = nixpkgs.lib.genAttrs supportedSystems;
      pkgsForSystem = system: import nixpkgs {
        inherit system;
        overlays = [ rust-overlay.overlays.default ];
      };
    in
    {
      packages = forAllSystems (system:
        let
          pkgs = pkgsForSystem system;
        in
        {
          default = pkgs.rustPlatform.buildRustPackage {
            pname = "spinning-rust-chiller";
            version = "0.1.0";
            src = ./.;

            nativeBuildInputs = with pkgs; [
              pkg-config
              makeWrapper
            ];

            buildInputs = [ pkgs.smartmontools ];

            postFixup = ''
              wrapProgram $out/bin/spinning-rust-chiller \
                --prefix PATH : ${pkgs.lib.makeBinPath [ pkgs.smartmontools ]}
            '';

            cargoLock = {
              lockFile = ./Cargo.lock;
            };
          };
        });

      nixosModules.default = { config, lib, pkgs, ... }:
        let
          cfg = config.services.spinning-rust-chiller;
          configFile = pkgs.writeText "spinning-rust-chiller.toml" ''
            hdds = [ ${builtins.concatStringsSep ", " (map (x: ''"${x}"'') cfg.hdds)} ]
            max_speed = ${toString cfg.maxSpeed}
            min_speed = ${toString cfg.minSpeed}
            temp_low = ${toString cfg.tempLow}
            temp_high = ${toString cfg.tempHigh}
            interval = ${toString cfg.interval}
            fan_control_path = "${cfg.pwmPath}"
          '';
        in
        {
          options.services.spinning-rust-chiller = {
            enable = lib.mkEnableOption "HDD temperature-based fan control";

            package = lib.mkOption {
              type = lib.types.package;
              description = "The spinning-rust-chiller package to use";
              default = self.packages.${pkgs.system}.default; # Reference to your flake's package
            };

            enableFile = lib.mkOption {
              type = lib.types.str;
              description = "Full path to fan PWM enable file";
              example = "/sys/class/hwmon/hwmon1/pwm7_enable";
            };

            hdds = lib.mkOption {
              type = lib.types.listOf lib.types.str;
              description = "List of HDD device paths to monitor";
              example = [ "/dev/sda" "/dev/sdb" ];
            };

            pwmPath = lib.mkOption {
              type = lib.types.str;
              description = "Full path to fan PWM control file";
              example = "/sys/class/hwmon/hwmon1/pwm7";
            };

            maxSpeed = lib.mkOption {
              type = lib.types.int;
              default = 255;
              description = "Maximum fan speed (PWM value 0-255)";
            };

            minSpeed = lib.mkOption {
              type = lib.types.int;
              default = 30;
              description = "Minimum fan speed (PWM value 0-255)";
            };

            tempLow = lib.mkOption {
              type = lib.types.int;
              default = 35;
              description = "Temperature at which fan runs at minimum speed (°C)";
            };

            tempHigh = lib.mkOption {
              type = lib.types.int;
              default = 50;
              description = "Temperature at which fan runs at maximum speed (°C)";
            };

            interval = lib.mkOption {
              type = lib.types.int;
              default = 10;
              description = "Monitoring interval in seconds";
            };
          };

          config = lib.mkIf cfg.enable {
            systemd.services.spinning-rust-chiller = {
              description = "HDD temperature-based fan control";
              after = [ "multi-user.target" ];
              wantedBy = [ "multi-user.target" ];

              serviceConfig = {
                ExecStart = pkgs.writeShellScript "start-spinning-rust-chiller" ''
                  echo "Starting fan control service..."

                  # Enable manual control if enable file exists
                  if [ -f "${cfg.enableFile}" ]; then
                    echo 1 > "${cfg.enableFile}"
                  fi

                  exec ${cfg.package}/bin/spinning-rust-chiller \
                    --config ${configFile}
                '';

                Environment = [
                  "RUST_LOG=info"
                ];

                Restart = "always";
                RestartSec = "10s";
              };
            };
          };
        };
    };
}
