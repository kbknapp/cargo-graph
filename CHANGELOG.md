<a name="v0.2.0"></a>
## v0.2.0 (2015-12-11)

#### New Features

* Can now include dependency versions in the nodes by using the -I command line flag ([77a3055](https://github.com/kbknapp/cargo-graph/commit/77a3055170f369c7ed39bacc1ae34cecb63d9a27), closes [#13](https://github.com/kbknapp/cargo-graph/issues/13))

#### Bug Fixes

*   fixes panic when dev deps are also optional build deps ([86792f92](https://github.com/kbknapp/cargo-graph/commit/86792f923b0796201a81aca3c5b9cd83968be50c), closes [#18](https://github.com/kbknapp/cargo-graph/issues/18))
*   fixes dep and edge propagation ([ba498ffa](https://github.com/kbknapp/cargo-graph/commit/ba498ffa18533fe8fcc44ffc4f64dc6b8c27d693), closes [#15](https://github.com/kbknapp/cargo-graph/issues/15))
*   removes duplicate edges ([68fe19cd](https://github.com/kbknapp/cargo-graph/commit/68fe19cd8cf4d5b76bb9d443525dc9a5454d5030))


<a name="v0.1.3"></a>
## v0.1.3 (2015-11-14)


#### Improvements

* **Errors:**  improves error handling ergonomics ([da0dde32](https://github.com/kbknapp/cargo-graph/commit/da0dde323cb9f5b84f928095bd64160ba3d9f5f7))

#### Bug Fixes

*   fixes building on windows due to upstream dep ([99eb7f9e](https://github.com/kbknapp/cargo-graph/commit/99eb7f9ed7c190243c31bc41b4f8c0400383530c))
* **Dev Deps:**  fixes a bug where dev deps are not properly filtered ([8661c2fc](https://github.com/kbknapp/cargo-graph/commit/8661c2fc21d66cae37a43baaa778498efeed8ec7), closes [#4](https://github.com/kbknapp/cargo-graph/issues/4))



<a name="v0.1.2"></a>
### v0.1.2 (2015-11-13)


#### Bug Fixes

*   fixes building on windows due to upstream dep ([99eb7f9e](https://github.com/kbknapp/cargo-graph/commit/99eb7f9ed7c190243c31bc41b4f8c0400383530c))
* **Dev Deps:**  fixes a bug where dev deps are not properly filtered ([8661c2fc](https://github.com/kbknapp/cargo-graph/commit/8661c2fc21d66cae37a43baaa778498efeed8ec7), closes [#4](https://github.com/kbknapp/cargo-graph/issues/4))

#### Improvements

* **Errors:**  improves error handling ergonomics ([da0dde32](https://github.com/kbknapp/cargo-graph/commit/da0dde323cb9f5b84f928095bd64160ba3d9f5f7))



<a name="v0.1.1"></a>
## v0.1.1 (2015-11-04)


#### Documentation

*   adds png examples ([c00eb4aa](https://github.com/kbknapp/cargo-graph/commit/c00eb4aa0981d83c0fd8ac7236323fab85c2cc42))
*   updates usage in the docs ([f1713761](https://github.com/kbknapp/cargo-graph/commit/f1713761b3d63ff96ff89939f1c59012036ffded))
*   adds crate docs ([d94bdca6](https://github.com/kbknapp/cargo-graph/commit/d94bdca603cbdb843ec77e26064d98dbc25ee965))
*   updates readme ([16e3bca4](https://github.com/kbknapp/cargo-graph/commit/16e3bca473accea028610aded420b7058a03ce3a))

#### Bug Fixes

*   fixes clippy finds ([5d8f7920](https://github.com/kbknapp/cargo-graph/commit/5d8f79202560ed7f9090d2cc6bdc853191c16bb0))



