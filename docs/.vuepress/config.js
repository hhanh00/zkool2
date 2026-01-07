import { viteBundler } from "@vuepress/bundler-vite";
import { defaultTheme } from "@vuepress/theme-default";
import { defineUserConfig } from "vuepress";
import { markdownImagePlugin } from "@vuepress/plugin-markdown-image";
import { markdownMathPlugin } from "@vuepress/plugin-markdown-math";
import { markdownHintPlugin } from "@vuepress/plugin-markdown-hint";
import { markdownExtPlugin } from "@vuepress/plugin-markdown-ext";
import { markdownChartPlugin } from '@vuepress/plugin-markdown-chart'
import { searchPlugin } from '@vuepress/plugin-search'

export default defineUserConfig({
  title: "Zkool Documentation",
  base: '/zkool2/',
  bundler: viteBundler(),
  theme: defaultTheme({
    sidebar: [
      // "/overview",
      {
        text: "Guides",
        prefix: "/guide/",
        children: [
          "start",
          "accounts",
          "sync",
          "account",
          "addresses",
          "pay",
          "other",
          "build",
        ],
      },
      {
        text: "Recipes",
        prefix: "/recipe/",
        children: [
          "account",
          "restore",
          "cold",
          "ledger",
          "sync",
          "folder",
          "database",
          "tor",
          "security",
          "iso",
          "mempool",
          "puri",
          "coin",
          "net",
          "csv",
        ],
      },
      {
        text: "Categories / Reports",
        prefix: "/report/",
        children: [
          "overview",
          "examples",
          "categories",
          "price",
          "chart",
        ],
      },
      {
        text: "MultiSig Accounts / FROST",
        prefix: "/frost/",
        children: [
          "overview",
          "dkg",
          "mpc",
        ],
      },
      {
        text: "GraphQL",
        prefix: "/graphql/",
        children: [
          "api",
          "build",
          "graphiql",
          "account",
          "sync",
          "balance",
          "history",
          "receive",
          "payments",
          "unconfirmed",
          "notifications",
        ],
      },
      {
        text: "Zcash tech",
        prefix: "/tech/",
        children: [
          "shielding",
          "bitcoin",
          "graph",
          "privacy",
        ],
      },
    ],
  }),
  plugins: [
    searchPlugin({}),
    markdownImagePlugin({
      figure: true,
      lazyload: true,
      size: true,
    }),
    markdownMathPlugin({}),
    markdownExtPlugin({
      gfm: true,
      breaks: false,
    }),
    markdownChartPlugin({
      mermaid: true,
    }),
  ],
});
