# Dilla WASM with Bindgen

## Install NPM package

Create a Gitlab [personal access token](https://gitlab.com/-/profile/personal_access_tokens) with scope `read_api`, `read_registry`

1. Authenticate to the Package Registry

```shell
npm config set -- //gitlab.com/api/v4/projects/44351519/packages/npm/:_authToken=_GITLAB_PERSONAL_TOKEN_
```

2. Set the registry

```shell
npm config set @dilla-io:registry=https://gitlab.com/api/v4/projects/44351519/packages/npm/
```

3. Install

```shell
npm install @dilla-io/javascript
npm install @dilla-io/swing_1
```
