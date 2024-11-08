{
  devenv.shells.default = {
    languages.rust.enable = true;

    pre-commit = {
      hooks = {
        clippy.enable = true;
        rustfmt.enable = true;

        alejandra.enable = true;
        nil.enable = true;
        statix.enable = true;

        typos.enable = true;
      };
    };
  };
}
