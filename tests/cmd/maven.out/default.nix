{
  lib,
  maven,
  fetchFromGitHub,
  nix-update-script,
}:

maven.buildMavenPackage (finalAttrs: {
  pname = "commons-lang";
  version = "3.20.0";
  __structuredAttrs = true;

  src = fetchFromGitHub {
    owner = "apache";
    repo = "commons-lang";
    tag = "rel/commons-lang-${finalAttrs.version}";
    hash = "sha256-6FivHu4P6fJ1cDABHLt6TXM3PJ08W/73KwPiPa5RONA=";
  };

  mvnHash = "sha256-n6zd+CCKVDH8M0MMFz5voCIhmWKx4FnGNKMOr1N5y4E=";

  passthru.updateScript = nix-update-script { };

  meta = {
    description = "[..]";
    homepage = "https://github.com/apache/commons-lang";
    license = lib.licenses.asl20;
    maintainers = with lib.maintainers; [ alice ];
    mainProgram = "commons-lang";
  };
})
