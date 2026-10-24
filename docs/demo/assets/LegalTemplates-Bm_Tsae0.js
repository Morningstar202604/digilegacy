import{E as e,G as t,N as n,a as r,b as i,c as a,gt as o,l as s,m as c,o as l,r as u,vt as d,w as f,x as p}from"./runtime-core.esm-bundler-DLYvfG9C.js";import{S as m,r as h,x as g}from"./index-YJvZd2Cu.js";var _={class:`legal fade-in`},v={class:`card mb-24`},y={class:`flex gap-8 flex-wrap`,role:`group`,"aria-label":`按文书类型筛选`},b=[`aria-pressed`],x=[`aria-pressed`,`onClick`],S={class:`card mb-24`},C={style:{padding:`12px 16px`,background:`var(--gray-50)`,display:`flex`,"justify-content":`space-between`,"align-items":`center`}},w={class:`flex items-center gap-8`},T={class:`tag tag-secondary`},E={class:`flex gap-8`},D=[`onClick`],O=[`onClick`],k={style:{padding:`16px`}},A={class:`text-muted`,style:{"font-size":`13px`,"line-height":`1.5`}},j={class:`mt-12`,style:{display:`flex`,"flex-wrap":`wrap`,gap:`6px`}},M={class:`modal-header`},N={id:`legal-preview-title`,style:{"font-size":`18px`,"font-weight":`600`,margin:`0`}},P={class:`modal-body`},F={style:{"white-space":`pre-wrap`,"font-size":`13px`,"line-height":`1.6`,color:`var(--gray-700)`}},I=h(c({__name:`LegalTemplates`,setup(c){let h=t(`all`),I=[{value:`will`,label:`遗嘱`},{value:`inheritance`,label:`继承声明`},{value:`power_of_attorney`,label:`授权委托书`},{value:`affidavit`,label:`宣誓书/保证书`},{value:`application`,label:`申请书`},{value:`agreement`,label:`协议书`}],L=[{id:`will-digital`,name:`数字资产遗嘱（标准版）`,type:`遗嘱`,description:`适用于将数字资产（账号、虚拟货币、数字内容等）纳入遗嘱的标准模板。`,tags:[`数字资产`,`遗嘱`,`标准`],content:`数字资产遗嘱

立遗嘱人：__________，身份证号：__________，住址：__________。

本人神志清醒，自愿订立本遗嘱，对本人名下数字资产作出如下处置：

一、数字资产清单
1. 社交媒体账号：__________（平台：__________）
2. 电子邮箱：__________
3. 数字钱包/加密货币：__________（地址：__________）
4. 云存储账号：__________（平台：__________）
5. 其他数字资产：__________

二、处置意愿
1. 上述数字资产由 __________（继承人姓名）继承
2. 如上述继承人先于本人去世，则由 __________ 继承
3. 本人希望 __________（平台/账号）被删除/纪念化

三、特别说明
1. 本遗嘱涉及的数字资产账号密码/密钥已另行保管，存放于 __________
2. 继承人应遵守各平台用户协议及法律法规
3. 本遗嘱自本人签字之日起生效

立遗嘱人（签字）：__________
日期：____年____月____日

见证人1（签字）：__________ 身份证号：__________
见证人2（签字）：__________ 身份证号：__________`},{id:`inheritance-declaration`,name:`数字遗产继承声明书`,type:`继承声明`,description:`继承人向平台/机构申请继承数字资产时使用的标准声明书。`,tags:[`继承`,`声明`,`平台申请`],content:`数字遗产继承声明书

声明人：__________，身份证号：__________，联系电话：__________。

被继承人：__________，身份证号：__________，于____年____月____日去世。

声明事项：
本人系被继承人的 __________（关系），根据《中华人民共和国民法典》相关规定，对被继承人名下以下数字资产享有继承权：

1. 平台名称：__________，账号：__________
2. 平台名称：__________，账号：__________
3. 其他数字资产：__________

本人承诺：
1. 上述信息真实有效
2. 继承的数字资产将合法使用，不用于违法活动
3. 如有其他继承人主张权利，本人愿承担相应法律责任

随附材料：
□ 死亡证明
□ 继承权公证书
□ 本人身份证
□ 其他：__________

声明人（签字）：__________
日期：____年____月____日`},{id:`power-of-attorney-digital`,name:`数字资产处理授权委托书`,type:`授权委托书`,description:`授权他人代为处理数字资产相关事务的委托书模板。`,tags:[`授权`,`委托`,`代理`],content:`数字资产处理授权委托书

委托人：__________，身份证号：__________，联系电话：__________。

受托人：__________，身份证号：__________，联系电话：__________。

委托事项：
委托人因 __________ 原因，无法亲自处理以下数字资产相关事务，特委托受托人代为办理：

1. 平台名称：__________，账号：__________
   授权事项：□ 密码找回 □ 资产转移 □ 账号注销 □ 其他：__________

2. 平台名称：__________，账号：__________
   授权事项：□ 密码找回 □ 资产转移 □ 账号注销 □ 其他：__________

授权期限：自____年____月____日至____年____月____日

受托人权限：
1. 代为提交申请材料
2. 代为签署相关文件
3. 代为领取相关款项/资产
4. 其他：__________

委托人（签字）：__________
日期：____年____月____日

受托人（签字）：__________
日期：____年____月____日`},{id:`affidavit-digital-asset`,name:`数字资产宣誓书`,type:`宣誓书/保证书`,description:`在法庭或公证处使用的数字资产相关宣誓书。`,tags:[`宣誓`,`法庭`,`公证`],content:`数字资产宣誓书

本人 __________，身份证号 __________，在此郑重宣誓：

1. 本人对 __________（被继承人）名下的以下数字资产享有合法继承权：
   - 平台：__________，账号：__________
   - 平台：__________，账号：__________

2. 本人提供的所有材料（死亡证明、公证书、身份证明等）均真实有效。

3. 本人承诺继承的数字资产将合法使用，不用于任何违法活动。

4. 如有虚假陈述，本人愿承担相应法律责任。

宣誓人（签字）：__________
日期：____年____月____日

见证人（签字）：__________
日期：____年____月____日`},{id:`platform-application`,name:`平台继承申请表（通用）`,type:`申请书`,description:`向各平台申请继承数字资产时使用的通用申请表模板。`,tags:[`申请`,`平台`,`通用`],content:`数字遗产继承申请表

致：__________（平台名称）

申请人信息：
姓名：__________，身份证号：__________，联系电话：__________，邮箱：__________。

被继承人信息：
姓名：__________，身份证号：__________，账号：__________。

申请事项：
本人系被继承人的 __________（关系），现申请继承被继承人在贵平台的以下数字资产：
1. 账号：__________
2. 资产类型：□ 余额 □ 会员权益 □ 数字内容 □ 其他：__________
3. 预估价值：__________

随附材料：
□ 被继承人死亡证明
□ 继承权公证书/法院判决书
□ 申请人身份证
□ 亲属关系证明
□ 其他：__________

申请人承诺：
1. 上述信息真实有效
2. 如有虚假，愿承担相应法律责任
3. 继承的资产将合法使用

申请人（签字）：__________
日期：____年____月____日`},{id:`inheritance-agreement`,name:`数字遗产分割协议书`,type:`协议书`,description:`多个继承人之间协商分割数字资产时使用的协议书模板。`,tags:[`分割`,`协议`,`多继承人`],content:`数字遗产分割协议书

甲方：__________，身份证号：__________，联系电话：__________。
乙方：__________，身份证号：__________，联系电话：__________。
丙方：__________，身份证号：__________，联系电话：__________。

被继承人：__________，于____年____月____日去世。

经各方协商一致，就被继承人名下数字资产分割达成如下协议：

一、数字资产清单
1. 平台：__________，账号：__________，预估价值：__________
2. 平台：__________，账号：__________，预估价值：__________
3. 其他数字资产：__________

二、分割方案
1. 甲方分得：__________
2. 乙方分得：__________
3. 丙方分得：__________

三、其他约定
1. 各方应互相配合办理相关手续
2. 如有争议，应友好协商解决
3. 本协议自各方签字之日起生效

甲方（签字）：__________ 日期：____年____月____日
乙方（签字）：__________ 日期：____年____月____日
丙方（签字）：__________ 日期：____年____月____日`}],R=r(()=>{var e;if(h.value===`all`)return L;let t=(e=I.find(e=>e.value===h.value))==null?void 0:e.label;return t?L.filter(e=>e.type===t):L}),z=t(null),B=t(null),V=t(null),H=null;function U(){z.value=null}function W(e){e.key===`Escape`&&U()}function G(e){let t=B.value;if(!t)return;let n=Array.from(t.querySelectorAll(`button:not([disabled]), input, [href], textarea, select`));if(n.length===0)return;let r=n[0],i=n[n.length-1],a=document.activeElement;e.shiftKey&&(a===r||!t.contains(a))?(e.preventDefault(),i.focus()):!e.shiftKey&&(a===i||!t.contains(a))&&(e.preventDefault(),r.focus())}n(z,async e=>{if(e){var t;H=document.activeElement,window.addEventListener(`keydown`,W),await i(),(t=V.value)==null||t.focus()}else{var n;window.removeEventListener(`keydown`,W),H==null||(n=H.focus)==null||n.call(H),H=null}}),p(()=>window.removeEventListener(`keydown`,W));function K(e){let t=new Blob([e.content],{type:`text/plain;charset=utf-8`}),n=URL.createObjectURL(t),r=document.createElement(`a`);r.href=n,r.download=`${e.name}.txt`,r.click(),URL.revokeObjectURL(n)}return(t,n)=>(f(),s(`div`,_,[n[3]||(n[3]=l(`h2`,{style:{"font-size":`22px`,"font-weight":`700`,"margin-bottom":`8px`}},`📜 法律文书模板库`,-1)),n[4]||(n[4]=l(`p`,{class:`text-muted mb-32`,style:{"font-size":`14px`}},` 数字遗产继承相关的法律文书模板，可直接下载使用或参考修改。 `,-1)),l(`div`,v,[n[1]||(n[1]=l(`div`,{class:`card-title mb-16`},`📂 按类型筛选`,-1)),l(`div`,y,[l(`button`,{class:o([`btn btn-sm`,h.value===`all`?`btn-primary`:`btn-secondary`]),"aria-pressed":h.value===`all`,onClick:n[0]||(n[0]=e=>h.value=`all`)},`全部`,10,b),(f(),s(u,null,e(I,e=>l(`button`,{class:o([`btn btn-sm`,h.value===e.value?`btn-primary`:`btn-secondary`]),key:e.value,"aria-pressed":h.value===e.value,onClick:t=>h.value=e.value},d(e.label),11,x)),64))])]),l(`div`,S,[n[2]||(n[2]=l(`div`,{class:`card-title mb-16`},`📋 模板清单`,-1)),(f(!0),s(u,null,e(R.value,t=>(f(),s(`div`,{key:t.id,class:`mb-16`,style:{border:`1px solid var(--gray-200)`,"border-radius":`8px`,overflow:`hidden`}},[l(`div`,C,[l(`div`,w,[l(`strong`,null,d(t.name),1),l(`span`,T,d(t.type),1)]),l(`div`,E,[l(`button`,{class:`btn btn-sm btn-secondary`,onClick:e=>z.value=t},`预览`,8,D),l(`button`,{class:`btn btn-sm btn-primary`,onClick:e=>K(t)},`下载`,8,O)])]),l(`div`,k,[l(`p`,A,d(t.description),1),l(`div`,j,[(f(!0),s(u,null,e(t.tags,e=>(f(),s(`span`,{key:e,class:`tag tag-secondary`,style:{"font-size":`11px`}},d(e),1))),128))])])]))),128))]),z.value?(f(),s(`div`,{key:0,class:`modal-overlay`,onClick:m(U,[`self`])},[l(`div`,{class:`modal-content`,role:`dialog`,"aria-modal":`true`,"aria-labelledby":`legal-preview-title`,ref_key:`previewRef`,ref:B,onKeydown:g(G,[`tab`])},[l(`div`,M,[l(`h3`,N,d(z.value.name),1),l(`button`,{class:`btn btn-sm btn-secondary`,ref_key:`previewCloseRef`,ref:V,onClick:U},`关闭`,512)]),l(`div`,P,[l(`pre`,F,d(z.value.content),1)])],544)])):a(``,!0),n[5]||(n[5]=l(`div`,{class:`card`,style:{background:`var(--gray-50)`}},[l(`div`,{class:`card-title mb-16`},`📖 使用说明`),l(`ul`,{style:{"font-size":`13px`,color:`var(--gray-600)`,"line-height":`2`,"padding-left":`20px`}},[l(`li`,null,`本模板库仅供参考，具体文书内容请根据实际情况修改`),l(`li`,null,`建议在律师指导下使用正式法律文书`),l(`li`,null,`各地公证处可能有自己的模板要求，请以当地要求为准`),l(`li`,null,`涉及重大财产继承时，建议委托专业律师处理`)])],-1))]))}}),[[`__scopeId`,`data-v-76d0c032`]]);export{I as default};