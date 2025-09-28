{
  lib,
  rustPlatform,
  pkg-config,
  dbus,
}:
rustPlatform.buildRustPackage {
  pname = "steam-dl-inhibit";
  version = "0.1.0";

  src = lib.cleanSource ./.;
  cargoLock = {
    lockFile = ./Cargo.lock;
  };

  buildInputs = [
    dbus
  ];
  nativeBuildInputs = [
    pkg-config
  ];

  meta.mainProgram = "steam-dl-inhibit";
}
