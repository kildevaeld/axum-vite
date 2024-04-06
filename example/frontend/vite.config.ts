import { defineConfig } from "vite";
import solid from "vite-plugin-solid";

export default defineConfig({
  build: {
    manifest: true,
  },
  plugins: [
    solid(),
    {
      writeBundle(_, options) {
        console.log(options);
      },
    },
  ],
});
