import { Button, NativeSelect, Text } from "@mantine/core";
import type { Key, Locale } from "./i18n";
const languages = ["zh","zh-Hans","zh-Hant","en","ja","ko","fr","de","es","pt","ru","it","ar","hi","th","vi"];
export function LanguageList({label,value,onChange,disabled,t,locale}:{label:string;value:string;onChange:(v:string)=>void;disabled:boolean;t:(key:Key)=>string;locale:Locale}) {
  const selected = value.split(",").filter(Boolean);
  const names = new Intl.DisplayNames([locale],{type:"language"});
  const includes = (code:string) => selected.some(x=>x.toLowerCase()===code.toLowerCase());
  const name = (code:string) => { try { return names.of(code)??code; } catch { return code; } };
  const save = (items:string[]) => onChange(items.join(","));
  function move(index:number, direction:number) {
    const items=[...selected]; [items[index],items[index+direction]]=[items[index+direction],items[index]]; save(items);
  }
  return <fieldset className="languages"><legend>{label}</legend>
    {selected.length===0&&<Text size="sm" c="dimmed">{t("mediaDefault")}</Text>}
    {selected.map((language,index)=><div className="language-row" key={index}>
      <NativeSelect aria-label={label+" "+(index+1)} disabled={disabled} value={language}
        data={[...new Set([...languages,language])].filter(x=>x===language||!includes(x)).map(code=>({value:code,label:name(code)}))}
        onChange={e=>{const items=[...selected];items[index]=e.currentTarget.value;save(items);}}/>
      <Button size="compact-sm" variant="subtle" aria-label={t("moveUp")+" "+(index+1)} disabled={disabled||index===0} onClick={()=>move(index,-1)}>↑</Button>
      <Button size="compact-sm" variant="subtle" aria-label={t("moveDown")+" "+(index+1)} disabled={disabled||index===selected.length-1} onClick={()=>move(index,1)}>↓</Button>
      <Button size="compact-sm" variant="subtle" aria-label={t("remove")+" "+(index+1)} disabled={disabled} onClick={()=>save(selected.filter((_,i)=>i!==index))}>{t("remove")}</Button>
    </div>)}
    <Button mt="sm" variant="default" disabled={disabled||selected.length>=8} onClick={()=>{const next=languages.find(x=>!selected.includes(x));if(next)save([...selected,next]);}}>{t("addLanguage")}</Button>
  </fieldset>;
}
