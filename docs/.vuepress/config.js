import { viteBundler } from "@vuepress/bundler-vite";
import { defaultTheme } from "@vuepress/theme-default";
import { defineUserConfig } from "vuepress";
import { markdownImagePlugin } from "@vuepress/plugin-markdown-image";
import { markdownMathPlugin } from "@vuepress/plugin-markdown-math";
import { markdownHintPlugin } from "@vuepress/plugin-markdown-hint";
import { markdownExtPlugin } from "@vuepress/plugin-markdown-ext";

export default defineUserConfig({
  title: "Zkool Documentation",
  base: '/zkool2/',
  bundler: viteBundler(),
  theme: defaultTheme({
    sidebar: [
      "/overview",
      {
        text: "Protocol",
        prefix: "/protocol/",
        children: [
          "concepts",
        ],
      }
    ],
  }),
  head: [["link", { rel: "stylesheet", href: "/main.css" }]],
  plugins: [
    markdownImagePlugin({
      figure: true,
      lazyload: true,
      size: true,
    }),
    markdownMathPlugin({}),
    markdownHintPlugin({}),
    markdownExtPlugin({
      gfm: true,
      breaks: false,
    }),
  ],
});
