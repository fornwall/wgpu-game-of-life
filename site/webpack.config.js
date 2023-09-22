import HtmlWebpackPlugin from "html-webpack-plugin";
import CopyWebpackPlugin from "copy-webpack-plugin";
import WorkboxPlugin from "workbox-webpack-plugin";

const plugins = [
  new HtmlWebpackPlugin({
    filename: "index.html",
    template: "index.html",
    chunks: ["home"],
  }),
  new CopyWebpackPlugin({
    patterns: [{ from: "static", to: "static" }],
  }),
];

if (process.env.NODE_ENV === "production") {
  plugins.push(
    new WorkboxPlugin.GenerateSW({
      // these options encourage the ServiceWorkers to get in there fast
      // and not allow any straggling "old" SWs to hang around
      cleanupOutdatedCaches: true,
      clientsClaim: true,
      skipWaiting: true,
      directoryIndex: "index.html",
      maximumFileSizeToCacheInBytes: 10_000_000,
    }),
  );
}

export default {
  entry: {
    home: "./index.js",
  },
  output: {
    publicPath: "/",
    assetModuleFilename: "asset-[name]-[contenthash][ext]",
    filename: "bundle-[name]-[contenthash].js",
    chunkFilename: "chunk-[name]-[contenthash].js",
  },
  devServer: {
    static: ".",
    headers: {
      "Cross-Origin-Embedder-Policy": "require-corp",
      "Cross-Origin-Opener-Policy": "same-origin",
    },
    port: 9090,
    // Disable webSocketServer to enable bfcache testing:
    // webSocketServer: false,
  },
  plugins,
  performance: {
    maxAssetSize: 10000000,
    maxEntrypointSize: 10000000,
  },
};
