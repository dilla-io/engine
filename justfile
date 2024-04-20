# https://github.com/casey/just
#
# Alternative to see: https://github.com/sagiegurari/cargo-make

# shellcheck disable=SC2086,SC1083,SC1036,SC1088

_default:
	@ just --list --unsorted

set dotenv-filename := ".env.default"
set dotenv-load

alias b := build
alias ba := build-all
alias c := check
alias pb := prebuild
alias pba := prebuild-all
alias r := run
alias rf := run-file

# DS := env_var_or_default("DS", "test")

# [chore] Installs the tools needed to develop
install-tools:
	cargo install cargo-binstall
	cargo binstall cargo-workspaces wasm-opt wasm-bindgen-cli cargo-component

# [chore] Upgrades the tools needed to develop
upgrade-tools:
	@ rustup update
	@ cargo update --quiet
	@ cargo binstall cargo-workspaces wasm-bindgen-cli cargo-component --force -y
	@ cargo binstall wasm-opt --force -y

# [chore] Update local Rust and tools
update:
	@ rustup update
	@ cargo update --quiet
	@ just check-versions

# [chore] Check tools versions
check-versions:
	@ rustc --version
	@ echo "  Latest: $(curl -L --silent "https://api.github.com/repos/rust-lang/rust/releases/latest" | grep -Po '"tag_name": "\K.*?(?=")')"
	@ cargo workspaces --version
	@ echo "  Latest: $(curl -L --silent "https://api.github.com/repos/pksunkara/cargo-workspaces/tags" | jq .[0].name -r)"
	@ wasm-bindgen --version
	@ echo "  Latest: $(curl -L --silent "https://api.github.com/repos/rustwasm/wasm-bindgen/releases/latest" | grep -Po '"tag_name": "\K.*?(?=")')"
	@ wasm-opt --version
	@ echo "  Latest: $(curl -L --silent "https://api.github.com/repos/WebAssembly/binaryen/releases/latest" | grep -Po '"tag_name": "\K.*?(?=")')"
	@ cargo component --version
	@ echo "  Latest: $(curl -L --silent "https://api.github.com/repos/bytecodealliance/cargo-component/releases/latest" | grep -Po '"tag_name": "\K.*?(?=")')"

# [code] Check code style and fix what's possible
check:
	@ cargo clippy --workspace --all-targets --all --fix --allow-dirty -- -D warnings -W clippy::all -A clippy::type_complexity
	@ cargo fmt --all

# [code] Generate Rust code documentation
doc:
	@ RUSTC_BOOTSTRAP=1 RUSTDOCFLAGS="--cfg=docsrs --html-in-header doc-header.html --enable-index-page -Z unstable-options" cargo doc --no-deps --document-private-items

# [code] Check Rust size
bloat:
	@ cargo bloat --crates -n 30

# [code] Remove cache, build and generated or build files
clean:
	@ cargo clean
	@ rm -rf ./dist ./var/run ./crates/dilla-renderer/build/ds.rs
	@ cp ./crates/dilla-renderer/src/build/test.rs ./crates/dilla-renderer/src/build/ds.rs

# [build] Use Pre-builder to generate DS folders in ./var/run from ./var/run_ds_src
prebuild ds:
	@./scripts/pre_build.sh run {{ds}} --skip-check --skip-pull

# [build] Use Pre-builder to generate ALL DS folders in ./var/run from ./var/run_ds_src
prebuild-all:
	@./scripts/pre_build.sh all --skip-check --skip-pull

# [build] Build with current DS, type can be cli, bg, co, ex
build ds type='bg':
	@./scripts/build-{{type}}.sh run {{ds}}

# [build] Build verbose one DS to ALL WASM
build-onev ds:
	@./scripts/build-cli.sh run {{ds}} -v
	@./scripts/build-bg.sh run {{ds}} -v
	@./scripts/build-co.sh run {{ds}} -v
	@./scripts/build-ex.sh run {{ds}} -v

# [build] Build one DS to ALL WASM
build-one ds:
	@./scripts/build-cli.sh run {{ds}}
	@./scripts/build-bg.sh run {{ds}}
	@./scripts/build-co.sh run {{ds}}
	@./scripts/build-ex.sh run {{ds}}

# [build] Build one DS to ALL WASM and sync to remote
build-one-sync ds:
	@./scripts/build-bg.sh run {{ds}}
	@./scripts/sync.sh bg {{ds}}
	@./scripts/build-co.sh run {{ds}}
	@./scripts/sync.sh co {{ds}}
	@./scripts/build-ex.sh run {{ds}}
	@./scripts/sync.sh ex {{ds}}

# [build] Build ALL DS, type can be cli, bg, co, ex or all
build-all type='all':
	@ if [ {{type}} == 'all' ]; then \
		./scripts/build-cli.sh all; \
		./scripts/build-bg.sh all; \
		./scripts/build-co.sh all; \
		./scripts/build-ex.sh all; \
	else \
		./scripts/build-{{type}}.sh all; \
	fi

# [build][sync] Build all and sync
build-all-sync:
	@./scripts/build-bg.sh all
	@./scripts/sync.sh bg_all
	@./scripts/build-co.sh all
	@./scripts/sync.sh co_all
	@./scripts/build-ex.sh all
	@./scripts/sync.sh ex_all

# [sync] Sync ds to remote, type can be bg, co, ex
sync ds type='bg':
	@./scripts/sync.sh {{type}} {{ds}}

# [sync] Sync all ds to remote
sync-all:
	@./scripts/sync.sh all_ds

# [test] All internal renderer tests
test-int test='':
	@ RUST_BACKTRACE=0 DS=test cargo test --no-default-features --package dilla-renderer -- --exact --nocapture {{test}}

# [test] All internal renderer tests coverage
test-int-cov out="Stdout":
	@ RUST_BACKTRACE=0 DS=test cargo tarpaulin \
		--skip-clean \
		--engine llvm \
		--workspace \
		--release \
		--ignore-tests \
		-e wasm-* \
		-e dilla-cli \
		--exclude-files **/bindings.rs \
		--exclude-files **/tests/** \
		--exclude-files **/var/** \
		--exclude-files **/xtask/** \
		--exclude-files **/crates/wasm-*/** \
		--exclude-files **/crates/dilla-cli/** \
		--exclude-files **/build.rs \
		--exclude-files **/build/** \
		--exclude-files **/tests/utils/mod.rs \
		--exclude-files **/dilla-renderer/src/main.rs \
		--exclude-files **/dilla-renderer/src/timing.rs \
		-o {{out}}

# [test] specific DS test (must have tests in ./run/DS/tests)
test ds:
	@./scripts/test.sh run {{ds}}

# [test] specific DS test with pre-build and build (must have tests in ./run_dr_src/DS/tests)
test-pb ds:
	@./scripts/pre_build.sh run {{ds}} --skip-check --skip-pull
	@./scripts/test.sh run {{ds}}

# [test](alias td) run doc tests without build
test-doc:
	@ RUST_BACKTRACE=0 DS=test cargo test --doc -p dilla-renderer -- --nocapture

# [test](alias td) run doc tests without build
test-doc-single name:
	@ RUST_BACKTRACE=0 DS=test cargo test --doc {{name}} -- --nocapture

# [test] run internal tests (only for DS=test)
tests:
	@ just test-int
	@ just test-doc

# [test] Generate tests files under run_ds_src
gen-tests ds:
	@./scripts/pre_build.sh run {{ds}} --skip-check --skip-pull
	@./scripts/build-cli.sh run {{ds}}
	@./scripts/test.sh gen {{ds}}

# [run] Render payload.json with output '_test_full' to terminal
run ds mode="_test_full":
	@ DS={{ds}} cargo run --quiet -- render payload.json -m "{{mode}}"

# [run] Render payload.json with output '_test' without prettify/minify to terminal
run-as-test ds mode="_test":
	@ DS={{ds}} cargo run --quiet -- render payload.json -m "{{mode}}" --raw

# [run] Render payload.json with output '_test_full' to file payload.html
run-file ds mode="_test_full":
	@ DS={{ds}} cargo run --quiet -- render payload.json -w payload.html -m "{{mode}}"

# [run] Render payload.json with output '_test' without prettify/minify to file payload.html
run-as-test-file ds mode="_test_full":
	@ DS={{ds}} cargo run --quiet -- render payload.json -w payload.html -m "{{mode}}" --raw

# [dev] Check swing and bootstrap 5 diff from var/run_ds_src
swing-diff:
	-@ diff -qr ./var/run_ds_src/bootstrap_5/components/ ./var/run_ds_src/swing_1/components/ | grep "bootstrap"

# [dev] Sync and copy bs5 templates to swing
swing-sync:
	@ cp -r ./var/run_ds_src/bootstrap_5/components/ ./var/run_ds_src/swing_1/

# [docker][LOCAL ONLY] Login to Docker.com registry with local script
docker-login:
	docker-login.sh -u dillaio -e docker.com

# [docker] Pull Dilla images locally
docker-pull:
	@ docker pull $DILLA_DOCKER_RUST
	@ docker pull $DILLA_DOCKER_SCHEMAS
	@ docker pull $DILLA_DOCKER_PREBUILDER

# [docker] SSH project with our Docker image
docker-ssh:
	@ docker run -it -v ./:/app -e CARGO_HOME=/app/.cargo -e GITHUB_TOKEN=$GITHUB_TOKEN -u 0 dillaio/docker bash