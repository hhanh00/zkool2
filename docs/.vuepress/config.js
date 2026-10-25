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
        text: "What's new",
        prefix: "/news/",
        collapsible: true,
        children: [
          "6.35", "6.34", "6.32", "6.31", "6.30", "6.29", "6.28", "6.27",
          "6.26", "6.25", "6.24", "6.23", "6.22", "6.21", "6.20", "6.19",
          "6.18", "6.17", "6.16", "6.15", "6.14", "6.13", "6.12", "6.11",
        ],
      },
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
          "contacts",
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
        text: "ZSA",
        prefix: "/zsa/",
        children: [
          "shielded-assets",
          "testnet",
          "using-zsa",
        ]
      },
      {
        text: "Coin Weighted Voting",
        prefix: "/vote/",
        children: [
          "overview",
          "howto",
        ],
      },
      {
        text: "Cloud Vault",
        prefix: "/vault/",
        children: [
          "overview",
          "howto",
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
