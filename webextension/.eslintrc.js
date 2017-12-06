module.exports = {
  "extends": [
    "eslint:recommended",
    "plugin:react/recommended"
  ],
  "plugins": [
    "react"
  ],
  "parser": "babel-eslint",
  "env": {
    "browser": true,
    "es6": true,
    "webextensions": true
  },
  "parserOptions": {
    "ecmaVersion": 8,
    "sourceType": "module",
    "ecmaFeatures": {
      "jsx": true
    }
  },
  "rules": {
    "brace-style": [
      "error",
      "1tbs"
    ],
    "curly": [
      "error"
    ],
    "indent": [
      "error",
      2
    ],
    "key-spacing": ["error"],
    "keyword-spacing": [
      "error",
      {
        "before": true,
        "after": true
      }
    ],
    "no-console": [
      0
    ],
    "no-multi-spaces": [
      "error"
    ],
    "no-trailing-spaces": [
      "error"
    ],
    "no-var": [
      "error"
    ],
    "prefer-template": [
      "error"
    ],
    "quotes": [
      "error",
      "single"
    ],
    "semi": [
      "error",
      "always"
    ],
    "space-before-blocks": ["error"]
  }
};
