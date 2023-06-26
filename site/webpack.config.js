import HtmlWebpackPlugin from "html-webpack-plugin";
import CopyWebpackPlugin from "copy-webpack-plugin";

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
    // Disable webSocketServer to enable bfcache testing:
    // webSocketServer: false,
  },
  plugins: [
    new HtmlWebpackPlugin({
      filename: "index.html",
      template: "index.html",
      chunks: ["home"],
    }),
    new CopyWebpackPlugin({
      patterns: [{ from: "static", to: "static" }],
    }),
  ],
  performance: {
    maxAssetSize: 10000000,
    maxEntrypointSize: 10000000,
  },
};
