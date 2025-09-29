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

  postInstall = ''
    mkdir -p $out/lib/systemd/user
    cp ${./systemd/steam-dl-inhibit.service} $out/lib/systemd/user/steam-dl-inhibit.service

    substituteInPlace $out/lib/systemd/user/steam-dl-inhibit.service \
      --replace-fail @@exe@@ $out/bin/steam-dl-inhibit
  '';

  meta.mainProgram = "steam-dl-inhibit";
}
