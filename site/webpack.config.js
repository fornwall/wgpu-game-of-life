import HtmlWebpackPlugin from "html-webpack-plugin";

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
  },
  plugins: [
    new HtmlWebpackPlugin({
      filename: "index.html",
      template: "index.html",
      chunks: ["home"],
    }),
  ],
};
