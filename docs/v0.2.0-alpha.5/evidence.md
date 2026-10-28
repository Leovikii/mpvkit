# alpha.5 发布证据

2026-10-28。用户已确认界面验证通过，并授权提交、推送、PR 合并、分支同步及公开发布。

- 源码提交：b4119c5（Improve setup）；[PR #12](https://github.com/Leovikii/mpvkit/pull/12) 合并为 3bcf323（Merge dev）。tag v0.2.0-alpha.5 固定此提交，不随发布记录移动。
- [PR CI](https://github.com/Leovikii/mpvkit/actions/runs/40684418540) 全部通过：Windows Rust 47 / Arch 52 项、各 5 专项默认忽略；Arch 包元数据另 1 项及隔离安装/重装/卸载通过。前端构建、2 项单元、17 项发布 mock 契约通过。
- [Release](https://github.com/Leovikii/mpvkit/actions/runs/40685498426) 双端测试、正式构建/签名、汇总和发布全部成功；公开 Release 从创建起 draft=false，prerelease=true，未设为 Latest。
- 六项资产均通过匿名公开地址下载；实际字节 SHA-256 与 GitHub asset digest、SHA256SUMS 一致。manifest 为 0.2.0-alpha.5，包含 windows-x86_64 / linux-x86_64，URL、签名与资产一致。
- 两端 Minisign 签名均使用发布环境公开公钥验证通过。匿名脚本、原始下载与结果保留本机 `.local/alpha5-public/`；不上传凭据或完整桌面截图。

[公开发布](https://github.com/Leovikii/mpvkit/releases/tag/v0.2.0-alpha.5) · [匿名更新 manifest](https://github.com/Leovikii/mpvkit/releases/download/v0.2.0-alpha.5/latest.json)

| 资产 | 字节数 | SHA-256 |
| --- | ---: | --- |
| MpvKit_0.2.0-alpha.5_x64-setup.exe | 4140550 | `68b6534eb9056be84a4fe1952a411d7b2e455a3db023fbd6ddab091e0a1256b9` |
| MpvKit_0.2.0-alpha.5_x64-setup.exe.sig | 456 | `003b3e62a1b075555566f6f224b12b1cbb90c3bd1d345149f205bb7e505693fa` |
| SHA256SUMS | 514 | `d5ed6234932f6032c78c5a53ddeb91f2d041cc8f2bf6dc706f9de8595759751c` |
| latest.json | 1636 | `13c200a71ae267d5c6cd1804351b39b983997cbce7d5e2272107bf102ea98440` |
| mpvkit-bin-0.2.0alpha5-1-x86_64.pkg.tar.zst | 7043553 | `c316384560e5fbf7ef4f136d97318feb272c402c0e77df7a8b2e40196c0998e7` |
| mpvkit-bin-0.2.0alpha5-1-x86_64.pkg.tar.zst.tauri.sig | 440 | `64f9d7959e056e5e6cbdce2ac76ea059361a20efcc7bfa45047163766f0eed21` |

用户视觉验收和本机 KDE 原生测试见[方案与交接](plan.md)。Windows 原生 GUI/WinGet/Mica/NSIS 跨版本更新、实际 Linux Polkit 安装/重开、宿主正式安装后的 KDE 菜单/任务栏视觉及多屏 DPI 等仍未验证；CI 和隔离载荷检查不替代这些实测。
