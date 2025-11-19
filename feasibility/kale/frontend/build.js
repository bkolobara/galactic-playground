const esbuild = require("esbuild");

const isWatch = process.argv.includes("--watch");

const buildOptions = {
  entryPoints: ["src/index.tsx"],
  bundle: true,
  outfile: "dist/bundle.js",
  platform: "browser",
  target: "es2020",
  loader: {
    ".tsx": "tsx",
    ".ts": "ts",
  },
  minify: !isWatch,
  sourcemap: isWatch,
  define: {
    global: "globalThis",
  },
  inject: ["./polyfills.js"],
};

if (isWatch) {
  esbuild.context(buildOptions).then((ctx) => {
    ctx.watch();
    console.log("Watching for changes...");
  });
} else {
  esbuild
    .build(buildOptions)
    .then(() => {
      console.log("Build complete!");
    })
    .catch(() => process.exit(1));
}
