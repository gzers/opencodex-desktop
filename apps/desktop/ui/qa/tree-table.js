import { createApp } from "vue"
import "../src/styles/tokens.css"
import "../src/styles/base.css"
import TreeTableFixture from "./TreeTableFixture.vue"

// Separate Vite QA entry: synthetic data only, no native API or production route.
createApp(TreeTableFixture).mount("#app")
