with import <nixpkgs> { };

mkShell {

  # Package names can be found via https://search.nixos.org/packages
  nativeBuildInputs = [
    SDL2
    SDL2_gfx
    SDL2_mixer 
    SDL2_ttf
    SDL2_image
    gnuplot
    valgrind
  ];
  buildInputs = [
    pkgs.pkg-config
    pkgs.openssl  # Example dependency for pkg-config
  ];
  NIX_ENFORCE_PURITY = true;

  shellHook = ''
  '';
}
