// A little bit simplified version
export const groupBy = <T, K extends keyof any>(arr: T[], key: (i: T) => K) =>
    arr.reduce((groups, item) => {
        (groups[key(item)] ||= []).push(item);
        return groups;
    }, {} as Record<K, T[]>);


export const volumeFormat = (num: number) => {
    const formatter = new Intl.NumberFormat('zh-CN', {
        notation: 'compact',
        minimumFractionDigits: 2,
        maximumFractionDigits: 2,
    });
    return formatter.format(num)
};


export const marketStr = (k: string) => {
    return k === 'sz' ? '深圳': k === 'sh' ? '上海': k;
}
