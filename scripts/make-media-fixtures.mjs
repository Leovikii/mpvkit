import fs from "node:fs/promises";
import path from "node:path";
import crypto from "node:crypto";
import {spawnSync} from "node:child_process";
const [ffmpeg,output]=process.argv.slice(2);
if(process.platform!=="win32" || !ffmpeg || !output || !path.isAbsolute(ffmpeg) || !path.isAbsolute(output)) throw Error("需要固定FFmpeg和新隔离输出目录的完整本地路径");
const tool=await fs.readFile(ffmpeg);
const digest=crypto.createHash("sha256").update(tool).digest("hex");
if(digest!=="7491af2d1d44f695d7c169db5f12b51d61667d585619dea75d3a0fc96215a5f1") throw Error("FFmpeg不匹配已校验研究工具，拒绝执行");
await fs.mkdir(output); // Never reuse or overwrite an existing fixture directory.
const definitions=[
 {id:"1080p-h264-sdr",width:1920,height:1080,encoder:"libx264",format:"yuv420p",gamma:"bt.1886",primaries:"bt.709",args:["-preset","ultrafast","-crf","32","-color_primaries","bt709","-color_trc","bt709","-colorspace","bt709"]},
 {id:"4k-hevc10-sdr",width:3840,height:2160,encoder:"libx265",format:"yuv420p10le",gamma:"bt.1886",primaries:"bt.709",args:["-preset","ultrafast","-crf","35","-x265-params","pools=4:frame-threads=2:log-level=error","-color_primaries","bt709","-color_trc","bt709","-colorspace","bt709"]},
 {id:"4k-hevc10-pq-tags",width:3840,height:2160,encoder:"libx265",format:"yuv420p10le",gamma:"pq",primaries:"bt.2020",args:["-preset","ultrafast","-crf","35","-x265-params","pools=4:frame-threads=2:log-level=error:colorprim=9:transfer=16:colormatrix=9:hdr10=1:hdr10-opt=1:master-display=G(13250,34500)B(7500,3000)R(34000,16000)WP(15635,16450)L(10000000,1):max-cll=1000,400","-color_primaries","bt2020","-color_trc","smpte2084","-colorspace","bt2020nc"]},
];
const cases=[];
for(const item of definitions){
 const file=path.join(output,item.id+".mp4");
 const result=spawnSync(ffmpeg,["-hide_banner","-nostdin","-loglevel","error","-n","-f","lavfi","-i","testsrc2=size="+item.width+"x"+item.height+":rate=24","-t","8","-an","-c:v",item.encoder,"-pix_fmt",item.format,...item.args,file],{windowsHide:true,timeout:120000,maxBuffer:1048576,stdio:["ignore","ignore","pipe"]});
 if(result.error || result.status!==0) throw Error(String(result.error??result.stderr));
 const bytes=await fs.readFile(file);
 cases.push({id:item.id,file,sha256:crypto.createHash("sha256").update(bytes).digest("hex"),width:item.width,height:item.height,metadata:{gamma:item.gamma,primaries:item.primaries}});
 console.log(item.id,bytes.length);
}
const manifest={schemaVersion:1,toolSha256:digest,toolSourceArchiveSha256:"a92d38a7d9d038db29eadc4a641821c0e5b85fa438aed28bb627458b3c3b1091",note:"合成测试图及PQ标签仅验证解码/参数/组件路径，不证明HDR亮度、色彩准确性、画质或功耗。",cases};
await fs.writeFile(path.join(output,"matrix.json"),JSON.stringify(manifest,null,2)+"\n");
console.log(path.join(output,"matrix.json"));
