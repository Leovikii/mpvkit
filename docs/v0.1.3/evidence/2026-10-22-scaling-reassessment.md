# 内置缩放与第三方着色器复核

日期：2026-10-22。范围：目标版本手册、源码、当前配置与第三方上游资料调查。没有执行新画质、HDR、掉帧或功耗对比；没有修改产品参数、安装着色器或操作真实播放器配置。

## 结论与决策边界

**继续采用 mpv 内置渲染能力作为核显笔记本到独显台式机的默认方案；不能把它表述为所有硬件、片源和显示组合都已证明具有最佳画质性价比。**

内置方案覆盖常规放大、缩小、色度重建和 HDR 映射，通用性与维护成本适合作为项目基线。第三方方案在 1080p 动画放大到 4K、特定模糊/压缩片源修复、色度边缘重建方面有实际研究价值。性能充裕、偏好更锐利线条的用户可能认为这些收益值得额外开销；这种偏好不能替代通用默认值，也不能被忽略。

本轮维持“不默认部署外部 shader”的产品决定，不新增增强菜单或修改三档参数。“省电、均衡、高画质”仍是策略名称，不是已测得的跨设备能效排名。下一步若投入对照测试，优先验证 **1080p SDR 动画 → 4K 的轻量 ArtCNN 与内置高画质**；其次才是用户确实认为过软的缩小场景。

## 目标版本与当前三档

当前固定 mpv 为 v0.41.0-1224-g6a262d11f（完整提交 6a262d11f63733d37e3ee6d5fafcb1be1ef83952）；既有运行记录中的 libplacebo 为 v7.377.0（v7.360.0-176-g285d1cc-dirty）。后者的完整对应构建源码仍有历史缺口，不能把当前上游 HEAD 当作这个二进制的精确源码。见[既有运行身份](../../v0.1.0/evidence/2026-10-20-m2-core.md)和[来源边界](../../v0.1.0/evidence/2026-10-20-current-regression.md)。

[项目参数表](../../../resources/profiles.json)通过原生命名 profile 应用，但不等于直接调用官方 fast/high-quality：

| 参数 | 省电 | 均衡 | 高画质 |
| --- | --- | --- | --- |
| 放大 scale | bilinear | lanczos | ewa_lanczossharp |
| 缩小 dscale | bilinear | hermite | hermite |
| scale-antiring | 0 | 0 | 0.6 |
| correct-downscaling | no | yes | yes |
| linear-downscaling | no | yes | yes |
| sigmoid-upscaling | no | yes | yes |
| hdr-peak-percentile | 0 | 0 | 99.995 |
| hdr-contrast-recovery | 0 | 0 | 0.30 |

固定[官方 builtin.conf](https://github.com/mpv-player/mpv/blob/6a262d11f63733d37e3ee6d5fafcb1be1ef83952/etc/builtin.conf)中的 high-quality 只覆盖 scale、scale-antiring 和上述两个 HDR 参数；gpu-hq 是废弃别名。官方 fast 还会关闭 dithering、HDR peak detection 并允许延迟检测，项目省电档没有照搬这些选项。因此不能借官方 fast 的行为推断项目省电档的功耗。

固定[默认值实现](https://github.com/mpv-player/mpv/blob/6a262d11f63733d37e3ee6d5fafcb1be1ef83952/video/out/gpu/video.c)确认 scale=lanczos、dscale=hermite、cscale 继承 scale。[选项手册](https://github.com/mpv-player/mpv/blob/6a262d11f63733d37e3ee6d5fafcb1be1ef83952/DOCS/man/options.rst)将 lanczos 描述为质量/性能平衡方案，将 EWA Lanczos 描述为相对慢、质量较好的方案；这不是目标设备的能效测量。

由此有三点直接影响结论：

- 均衡与高画质使用同一缩小算法。4K → 1080p 切高画质不能描述为“切换更高级的亮度缩小算法”，但色度处理和需要时的 HDR 映射仍可能改变。
- 当前没有独立设置 cscale。4:2:0 片源即使按亮度原尺寸显示，仍存在色度重建；“不需亮度放大”不等于完全没有可改善的渲染阶段。
- 高画质并非所有片源都更准确。锐度、振铃、纹理、降噪与主观观感互有取舍；省电档的 bilinear 则是明确的质量折中，不能同时称为画质最佳。

## 内置与第三方究竟差在哪里

两者通常都由 GPU shader 执行。区别是 mpv/libplacebo 自带的重采样与色彩处理，和通过 glsl-shaders 插入的额外算法；不是“有没有使用 GPU”，也不是“内置只支持简单拉伸”。

| 方案 | 主要处理阶段与潜在收益 | 代价、限制及项目判断 |
| --- | --- | --- |
| 内置 Lanczos / EWA Lanczos / Hermite | 通用重采样；抗振铃、缩小滤波范围等已有控制 | 不会凭空恢复丢失的真实细节；质量已足以作为基线，但不能宣称全场景第一 |
| ArtCNN C4F16 / C4F32 | 面向动画的学习式亮度 2 倍重建；无后缀版本定位中性，DS/DN 另带锐化/降噪取向 | 多层计算，实际性能随硬件变化；C4F16 值得作为最小对照候选。不要把主要面向离线任务的 R 系列或实验模型当实时默认 |
| FSRCNNX | 学习式超分辨率，发布说明称可缓解轻度压缩伪影 | 不能仅凭老版本社区推荐当作今天最优；LineArt 模型上游明确提示在自然内容上可能呈油画感。训练用 TensorFlow/CUDA 不等于 mpv 使用 GLSL 必须安装它们 |
| RAVU / ravu-lite / ravu-zoom | 受 RAISR 启发的方向/结构感知重建；有亮度、RGB/YUV 与任意倍率变体 | 不同变体不可混为一个性能档。上游明确 ravu-zoom 在目标分辨率计算，精确 2 倍放大时可能明显慢于 ravu/ravu-lite |
| Anime4K | 动画线条恢复、去模糊、降噪、放大；可提供更明显的锐利观感 | 不只是忠实缩放。上游列出振铃、色带、纹理模糊和过锐风险；A/B/C 与叠加模式适用条件不同，不作为电影/HDR 通用增强 |
| SSimDownscaler | 缩小后按局部统计调整，可能使细节/对比看起来更清楚 | 不能恢复缩小必然丢失的分辨率。当前上游注明针对 linear-downscaling=no 调整，与项目均衡/高画质的 yes 不同；需对整条链对照，不能直接追加并称为无副作用升级 |
| KrigBilateral / ArtCNN Chroma | 色度重建，改善某些彩色边缘或色度损伤 | 改善不是全图亮度超分。ArtCNN Chroma 明确面向 4:2:0 BT.709，且要求先 bilinear 色度放大，不能直接推广到 BT.2020/PQ HDR |

Anime4K 的 README 主打原生 1080p 动画到 4K；更细的模式说明也讨论其他降质场景。这说明需要按具体模型和模式判断，不能概括为“所有低分辨率动画都适用”或“只能用于一种分辨率”。其更黑、更细线条的感知收益也不能自动等同于还原原画。

这些着色器一般是单帧空间处理，并不自动增加运动信息或把 24 fps 变成真实 144 fps。真实视频还应检查线条移动时的闪烁、细纹稳定性、噪点/胶片颗粒、字幕和场景切换，不能只看暂停截图。

## 屏幕与片源场景矩阵

下表是机制与资料支持的推荐，不是已测设备排名。以实际视频显示矩形、裁剪及像素比例计算倍率，不能只看桌面分辨率或 GUI 中选择的目标分辨率。HDR 能力也独立于分辨率和刷新率。

| 片源 → 实际输出 | 首要问题 | 推荐及例外 |
| --- | --- | --- |
| 4K HDR 电影 → 4K HDR 屏 | 正确 HDR 输出/目标亮度与色域、硬解、呈现稳定性；通常无需亮度放大 | 内置链优先。超分通常无对应问题可解决；仍有色度重建及可能的亮度映射，不能将所有 HDR 输出都称为零处理直通 |
| 4K HDR 电影 → 4K SDR 屏 | HDR→SDR 映射、亮部/暗部与色域处理 | 优先内置色彩链及正确目标设置；第三方超分不能替代 tone mapping |
| 4K HDR 电影 → 1080p SDR / HDR 屏 | 缩小；SDR 需 HDR→SDR，HDR 屏仍需匹配其能力 | 内置均衡/高画质为默认。若觉得过软，先比较内置缩小滤波器；有明确剩余问题才测试 SSimDownscaler，不能用亮度超分当缩小增强 |
| 1080p SDR 动画 → 4K 屏 | 亮度约 2 倍放大、斜线与细线重建 | 内置高画质是稳妥起点；这是第三方最有价值的对照场景。ArtCNN/RAVU/FSRCNNX 可比较；Anime4K 适合明确接受修复/锐化风格的用户 |
| 1080p 真人电影 → 4K 屏 | 放大与保留自然纹理/胶片颗粒 | 内置优先；学习式超分可按片源试验，不默认使用动画线条修复或强降噪 |
| 1080p 动画 → 1080p 屏 | 原尺寸亮度、色度重建、已有压缩/模糊 | 无需为了亮度缩放先升到 4K 再缩回。针对降质的恢复可有主观收益，但属于可选修复，不是普遍必要步骤 |
| 1080p → 1440p、窗口播放等非整数倍率 | 真实倍率、预放大后再缩小的总链路 | 内置通常更省事。2 倍模型可能还要缩回目标尺寸，收益和成本不能按 1080p→4K 直接外推 |

缩小的“更锐”与“更正确”不是同义词：更强高频保留可能带来振铃、混叠和移动细纹闪烁；较软的 Hermite 也有相应取舍。官方默认选择不能替代个人观感比较，第三方优化某个指标也不自动证明全片更好。

## 高刷、核显与独显的判断

4K144 屏播放 24 fps 视频，并不意味着每秒完整运行 144 次解码与超分。缓存、重复呈现、同步和插值要分开看。项目当前 video-sync=audio、interpolation=no；继承[此前高刷研究](../../v0.1.2/evidence/2026-10-22-display-sync.md)，不为本次调查改变同步策略，也不伪造 display-fps。

源视频 24/60 fps 分别约每 41.67/16.67 ms 产生一帧；144 Hz 的呈现间隔约 6.94 ms。这些是不同层次的预算，不能混为单个 shader 必须满足的跨设备硬阈值。24 fps 在理想 120/144 Hz 下可整数重复，在 60 Hz 下可能呈 3:2 节奏；实际 23.976 Hz、系统同步及驱动仍需核对。缩放着色器不会消除所有运动顿挫。

- 核显笔记本：优先内置均衡；电池、风扇、温度或掉帧约束下选择省电。现代核显也可能承担轻量重建，但不能由“核显”一词断言不能用，或由瞬时零掉帧推断续航可接受。
- 独显台式机：内置高画质是合理候选；如主要看 1080p 动画且输出 4K，可比较轻量超分。独显有算力不意味着额外功耗、风扇和伪影收益必然值得。
- 4K60 HDR、倍速、字幕/OSD、跨屏切换比单一 1080p24 暂停图有不同压力；硬解是否成功、显存/共享带宽、呈现后端及电源模式都可能改变结论。

“性价比”至少包含可见收益、忠实度、平滑性、功耗/噪声和维护成本。GPU 占用百分比、厂商档次、shader pass 数量或价格都不能独立代替这些测量。

## 公开对比能支持什么

[Artoriuz 的 mpv 重采样对比](https://artoriuz.github.io/blog/mpv_upscaling.html)提供了 ArtCNN、RAVU 与内置滤波器的可复核方法。当前页面的单张动画亮度图测试中，ArtCNN_C4F16/C4F32、RAVU 的多项重建指标高于内置参考，足以反对“第三方不会有收益”的绝对说法。

但作者自己明确指出只有一张测试图，并解释人工降采样方法与训练数据匹配会偏向某些模型；作者也同时开发 ArtCNN。因此这既不是独立多片源盲测，也没有证明目标核显/独显上的实时功耗收益。PSNR、SSIM 和主观锐利感并不总是同向，不能把该榜单直接变成 MpvKit 默认配置。

本轮没找到并核实一套能覆盖目标固定构建、两类硬件、真实 1080p/4K、SDR/HDR、60 Hz/高刷及功耗的统一对照证据。结论应保持推荐性质，不能升级为“跨全部设备实证最优”。

## 最小后续验证

这是一份后续入口，不是本轮已完成的播放实验，也不新增自动化或发布流程。

1. 在可用设备上固定 mpv、后端、驱动、电源模式、真实输出矩形、系统 HDR 与屏幕亮度；记录实际硬解、色彩输出及每个 shader 的身份、触发条件和执行阶段。
2. 先比较内置均衡、内置高画质及一个无 DS/DN 后缀的 ArtCNN_C4F16，选 1080p SDR 动画到 4K，包含细线、平移、暗部渐变、纹理与字幕。保留随机顺序的正常观看距离动态 A/B；C4F32、Anime4K 或其他模型仅在出现明确需要后扩展。
3. 缩小另设 4K→1080p 对照，先比较内置 Hermite 与较锐的内置滤波器，再决定是否测试 SSimDownscaler。HDR 与 SDR 分开，不能拿改变线性处理后的结果只归功于 shader。
4. 排除初始化/着色器编译预热后，观察足够长的热稳定播放，重复测量渲染/呈现耗时及尾部波动、解码/呈现掉帧、音视频偏差、整机能耗或一致条件下的电池消耗、温度/风扇；切屏与倍速单独记录。
5. 只有收益可复现、动态伪影可接受、持续无异常掉帧且能耗可接受，才提出针对该场景的可选增强。核显本和独显机、实体 HDR/高刷不可用时明确留空，不使用更改软件标签或截图替代。

## 来源与复核记录

第三方以下固定提交/修订只标识本轮阅读对象，不表示已选为产品组件或通过兼容/许可分发验收。

- [mpv 固定手册](https://github.com/mpv-player/mpv/blob/6a262d11f63733d37e3ee6d5fafcb1be1ef83952/DOCS/man/options.rst)：scale/cscale/dscale、correct-downscaling、sigmoid-upscaling、glsl-shaders、tone-mapping、HDR 目标、插值与同步。手册明确 linear-downscaling 对 HDR 内容无效果，不把该开关作为 HDR 缩小质量承诺。
- [gpu-next 参数映射](https://github.com/mpv-player/mpv/blob/6a262d11f63733d37e3ee6d5fafcb1be1ef83952/video/out/vo_gpu_next.c)：缩放、色度、帧混合、峰值检测与色调映射是不同阶段。手册对 tone-mapping=auto 的说明与二进制实际行为仍应通过播放核对，不凭最新 libplacebo 推断固定构建。
- [ArtCNN README](https://github.com/Artoriuz/ArtCNN/blob/e753502f647afdd4e80c2d7aba88a7c289ec8335/README.md)及[C4F16 shader](https://github.com/Artoriuz/ArtCNN/blob/e753502f647afdd4e80c2d7aba88a7c289ec8335/GLSL/ArtCNN_C4F16.glsl)：实时/离线架构、后缀、色度适用条件；所读 shader 头部为 MIT。
- [FSRCNN-TensorFlow README](https://github.com/igv/FSRCNN-TensorFlow/blob/ea173caa522924e75b5ad0ae5c89d0595ec5d495/README.md)与[FSRCNNX 1.1 发布说明](https://github.com/igv/FSRCNN-TensorFlow/releases/tag/1.1)：训练环境和实际播放 shader 分开，LineArt 警告来自发布说明。发布说明可变，本地保留本次 API 响应。
- [RAVU 上游](https://github.com/bjin/mpv-prescalers/blob/b3f0a59d68f33b7162051ea5970a5169558f0ea2/README.md)：算法变体、阶段、性能提示与 LGPLv3 说明。
- [Anime4K README](https://github.com/bloc97/Anime4K/blob/7684e9586f8dcc738af08a1cdceb024cc184f426/README.md)与[模式说明](https://github.com/bloc97/Anime4K/blob/7684e9586f8dcc738af08a1cdceb024cc184f426/md/GLSL_Instructions_Advanced.md)：用途、修复机制及副作用；历史 Vega64 示例不能作为当前全硬件性能依据。
- [SSimDownscaler 固定修订](https://gist.github.com/igv/36508af3ffc84410fe39761d6969be10/38992bce7f9ff844f800820df0908692b65bb74a)：POSTKERNEL/PREKERNEL 钩子、额外处理及线性缩小前提；所读文件为 LGPL-3.0-or-later。
- [KrigBilateral 固定修订](https://gist.github.com/igv/a015fc885d5c22e6891820ad89555637/038064821c5f768dfc6c00261535018d5932cdd5)：CHROMA/LUMA 处理；所读文件署名 Shiandow，托管于 igv gist，不能将托管者直接当作唯一作者，许可为 LGPL-3.0-or-later。

网络检索连接器未返回结果，改用 HTTPS 直接读取以上上游资料；普通命令环境发生 I/O 错误后使用获准的沙箱外命令完成读取。原始资料保留在仓库忽略目录 `.local/shader-research-2026-10-22`，没有执行下载的源码或 shader。公开资料属于技术依据，不冒充本机新测量。

本轮只新增本文并追加当前计划交接；保留开始时已有未提交改动。验证限定于链接、UTF-8/LF、格式、范围一致性和本轮差异；既有短时图形测试仅证明原范围可运行，不能升级为这次画质/功耗比较。实体设备与完整发布门槛保持未完成。

## 公开测试证据补充（同日追问）

用户追问公开测试能否支持结论。本次联网检索成功，新增核对一份作者实测，并区分实测、上游设计说明与项目推论；不是重跑公开测试。

- [Artoriuz 画质测试](https://artoriuz.github.io/blog/mpv_upscaling.html#upsampling_results)：单张动画亮度图经 box 缩到一半再重建，Lanczos 的 PSNR/SSIM 为 36.5113/0.9799，RAVU-lite-ar-r4 为 39.6571/0.9878，ArtCNN_C4F16 为 42.6808/0.9916，C4F32 为 43.4749/0.9923。只证明该样本/退化方法下重建指标更好，不是完整影片、HDR 或能耗排名；作者也是 ArtCNN 开发者。
- [UTUMI Hirosi 的公开测试](https://utuhiro78.github.io/linuxplayers/mpv_01.html)：页面标注 2026-10-09；Ryzen 5 5600G 核显，将 25 秒视频预缩到 480p，再以 200% 窗口和无等待模式处理。整段耗时：Lanczos 3.20 秒，ravu-lite-r4 3.04 秒，ravu-lite-ar-r4 3.16 秒，Anime4K Upscale_Denoise x2 S 3.79 秒，FSRCNNX x2 8-0-4-1 9.25 秒，ArtCNN_C4F16 13.99 秒。此处 Anime4K 是单个 shader，不是完整 A/B 模式链。
- 已只读检查[计时脚本](https://github.com/utuhiro78/linuxplayers/blob/main/images/mpv/mpv_shader_benchmark.py)：使用 no-config、window-scale=2、untimed、display-desync，测整个 mpv 子进程墙钟时间；没有在脚本中固定 vo/gpu-api、暖机重复或核验 shader 生效。缺少完整驱动/构建/执行阶段证据与功耗测量，因此只作为有限的作者报告。不能把 3.04 对 3.20 秒的小差距判为稳定胜出，也不能用低于片长证明每帧均按时呈现，更不能外推到 1080p→4K 或电池续航。
- 这份核显数据削弱“第三方必定昂贵”的笼统说法，同时提示较复杂 CNN 会有明显成本。内置默认策略依然是通用产品取舍，不是公开横评已经证明的全设备最优值。
- [mpv 官方手册](https://mpv.io/manual/stable/#options-scale)将 Lanczos 定位为画质与性能的良好平衡；[Anime4K 模式说明](https://github.com/bloc97/Anime4K/blob/7684e9586f8dcc738af08a1cdceb024cc184f426/md/GLSL_Instructions_Advanced.md)列出修复的副作用。这两项属于设计依据，不包装成跨硬件实测。

新增页面与脚本只读保存在原研究缓存目录，未执行下载脚本。本轮没有取得覆盖核显/独显、1080p/4K、HDR/SDR、高刷与整机功耗的统一实测。结论分级为：内置适合通用默认有官方依据；特定动画超分可优于传统插值有有限量化证据；“内置全场景最佳性价比”仍无足够证据。

## HDR 参数解释补充（同日追问）

已按固定 mpv 选项/参数映射，以及二进制版本字符串所指上游 libplacebo 提交 285d1cc 的公开源码核对。后者有既有 dirty 构建差异，以下为上游语义依据，不冒称已核对二进制完整对应源码或新做 HDR 实测。

- [峰值与对比恢复接口说明](https://github.com/haasn/libplacebo/blob/285d1cc/src/include/libplacebo/shaders/colorspace.h)：percentile=0 或 100 都按最亮像素测峰值；不是第 0 百分位，也不是关闭峰值检测。99.995 采用亮度分布的高百分位，减少极少数异常高光对全局映射的影响，有牺牲极亮细节的可能。
- [直方图实现](https://github.com/haasn/libplacebo/blob/285d1cc/src/shaders/colorspace.c)：仅 0<percentile<100 时启用直方图；高画质 99.995 有额外统计成本，0 避免该项成本，但峰值/平均亮度检测仍可能运行。真正控制峰值检测的是 hdr-compute-peak，项目未覆盖其 auto 默认。
- contrast-recovery=0 关闭局部对比恢复；0.30 是算法恢复强度，不是亮度、对比度或画质增加 30%。该算法利用低频特征与高频信息补偿 tone mapping 后的局部细节，可能提高主观锐度，也可能振铃或过锐。
- [特征图路径](https://github.com/haasn/libplacebo/blob/285d1cc/src/renderer.c)：对比恢复需要额外特征提取、低通与中间纹理；HDR 源无需向较低目标亮度适配时会跳过该路径。0.30 改为 0.10 不意味着成本降为三分之一，关闭到 0 才能跳过相应流程。
- 普通 SDR 动画不以这两项改善；HDR→SDR 或高亮 HDR→较低亮度 HDR 屏更可能体现其收益。显示器标注 HDR 不代表无需映射，是否实际执行还取决于元数据、目标能力和渲染路径。没有测定这两项在目标硬件上的毫秒或瓦数变化。

## HDR 直通与显示器映射（同日追问）

用户询问交给显示器处理是否更好。固定 mpv 手册明确区分 target（默认，按目标显示能力适配）、source（传统 HDR 直通，按源元数据输出，交由合成器/显示器映射）及实验性 source-dynamic。直通不承诺压缩视频或像素逐位不变，仍有解码、色度重建、缩放、字幕合成和系统输出链。source-dynamic 不发送完整 HDR10+/Dolby Vision 动态元数据。

当前项目的 HDR On 只写 target-colorspace-hint=yes、target-trc=pq、target-prim=bt.2020，没有写 target-colorspace-hint-mode，因此默认仍为 target；不能称作明确的 source 直通。其他 target 参数、ICC 与实际协商输出格式仍可能影响结果。HDR 输出模式与三档画质应按职责理解，切高画质不等于切换直通。

[Microsoft SetHDRMetaData 文档](https://learn.microsoft.com/en-us/windows/win32/api/dxgi1_5/nf-dxgi1_5-idxgiswapchain4-sethdrmetadata)明确不保证交换链元数据会送到显示器，各显示器的处理也不一致，并建议应用映射到显示器支持的色域与亮度范围。[Windows Advanced Color 架构](https://learn.microsoft.com/en-us/windows/win32/direct3darticles/high-dynamic-range)说明 DWM 合成与输出色彩转换，因此不能把 source 模式描述成完全绕过 Windows。固定 mpv 手册本身也指出部分场景由播放器按目标能力映射更好。

结论保持条件式：显示器对 HDR10 输入的映射优良、模式正确且链路验证通过时，source 可成为更合适的选择；目标亮度较低、设备映射较差或需要播放器逐场景控制时，target 可能更好。显示器更了解自身面板能力是可行性理由，不是该型号实测胜出的证据。减少播放器映射可能降低部分 GPU 工作，但不保证零处理或整机能耗更低。HDR→SDR 仍需映射。

本轮只查文档/源码并记录解释，未改配置或新增直通选项。后续若比较，应在固定实际 HDR 输出、片源、屏幕模式与校准条件下对照 target/source，观察高光细节、暗部、肤色、亮度稳定性及渲染/功耗；源/输出统计和显示器 HDR 标志只能验证部分链路，不证明原始元数据完整送达。实体测试与完整构建来源缺口保留。
