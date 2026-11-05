export function searchAddonGet(v,k){return v?.[k];}
export function searchAddonFlag(v,k){return v[k];}
export function searchAddonRegex(v){return v.regex;}
export function searchAddonRefresh(v){return {...v,incremental:true};}
export function searchAddonLimitReached(v,n){return n>=v;}
export function searchAddonLimitEnd(v,n){let x=+v;if(Number.isNaN(x))x=0;x=Math.trunc(x);return x<0?Math.max(n+x,0):Math.min(x,n);}