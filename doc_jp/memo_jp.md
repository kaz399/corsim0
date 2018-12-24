## corsim0 開発メモ

Rust初心者がいきなり無謀にCPUシミュレータを実装する記録

### Dec 25, 2018

Cでよくやる「引数に関数ポインタを渡して条件が成立したら関数を実行する」みたいな処理を作ろうとしたけどよくわからず、マクロで対処。

命令デコードが不毛に感じられてきたので、デコードをやりやすくするためにbitdecode.rsを作った。

capture = parse_bit(0b00011011, "aabbbbaa").unwrap()

みたいな感じで、第一引数にビットコード、第二引数にフォーマット文字列を指定して呼び出すと
aやbの文字位置のビットを集めたを持ったハッシュマップが返ってくる。　　
上の例だとこんな感じ。　　

capture["a"] = 0b0011  
capture["b"] = 0b1110  

#### フォーマット文字列のきまり

* ハッシュのキーに使えるのは一文字だけ

  自由な名前をつけられるとかっこいい気がするけど、そこまで頑張る必要を感じなかった。

* ' ', '_', '|'　はスペーサー

  32bitくらいの長さになると、適当な場所に区切りを入れられないと見づらい。

```
        let capture = parse_bit(&bitcode, "aaaabbbb").unwrap();
        assert_eq!(capture["a"], 0b1100);
        assert_eq!(capture.get("b"), Some(&0b1110));
```


### Dec 17, 2018

&selfの使いどころについて理解した気がする。

### Dec 12, 2018

main.rsやlib.rsで正しく
    
    mod なんとか

できていないと、他のファイル間での use がエラーになってうまくいかないということを知った。

プログラミング Rustを買った。

### Dec 09, 2018

Rust 2018にしてみた。

### Dec 04, 2018

テスト用バイナリを変更。  
もともとテストに使っていたものはライセンスの関係でリポジトリに含められないので
cortex-m-quickstartのサンプルをCortex-M0用にビルドしたものにした。

https://github.com/rust-embedded/cortex-m-quickstart

新テスト用バイナリで試すと１命令目で未実装になってしまって残念だった。


### Nov 28, 2018

ちょっとテストも書いた。  
Rust勉強会面白かった。


### Nov 15, 2018

#### デバイスと全体メモリマップ（２）

system構造体としてCPUとメモリをまとめた。
system.reset() とかできるようになったのでうれしい。

CPUから統一的にメモリアクセスができるようになった。

これでようやくメモリからのロード命令を作れる。

##### 今日の気づき

* 所有権がなくなっても構わないところでは素直に所有権を譲渡してしまえばよいようだ
* スタティックメソッド呼び出しは'.'ではなく'::'（相当に恥ずかしい間違い・・）

### Nov 13, 2018

#### デバイスと全体メモリマップ

デバイスができたら、デバイスを寄せ集めてシステム全体のメモリマップを組む。

device.rsにSystemMapを実装。

デバイスを作って空のSystemMapにregister_device()で追加していく感じ。

ライフタイムを書かないとエラーになるので適当に書いたが、まだいまいちよくわからない。

```  rust
    let mut rom: device::MemoryMappedDevice = device::MemoryMappedDevice {
        name: "ROM".to_string(),
        data: Box::new([0; RAMSIZE]),
        mapping: device::DeviceMapping {
            adrs: ROMADDR,
            size: ROMSIZE,
        },
        readable: true,
        writable: false,
    };

    let mut ram: device::MemoryMappedDevice = device::MemoryMappedDevice {
        name: "RAM".to_string(),
        data: Box::new([0; RAMSIZE]),
        mapping: device::DeviceMapping {
            adrs: RAMADDR,
            size: RAMSIZE,
        },
        readable: true,
        writable: true,
    };

    let mut system_map: device::SystemMap = device::SystemMap { map: Vec::new() };

    system_map.register_device(&mut ram);
    system_map.register_device(&mut rom);
```  

### Oct 16, 2018

#### デバイス

今まで適当にRAMとROMを扱っていたけど、そろそろちゃんと周辺デバイスというものを考えないといけない気がしてきたのでdevice.rsで実験。

とりあえずこんな形になった。

``` rust :device.rs
#[derive(Debug)]
pub struct DeviceMapping {
    pub adrs: u32,
    pub size: usize,
}

#[derive(Debug)]
pub struct Memory {
    pub data: Box<[u8]>,
    pub mapping: DeviceMapping,
}
```

``` rust :main.rs
use std::boxed::Box;

mod device;

const RAMSIZE: usize = 128;

fn main() {
    let mut ram: device::Memory = device::Memory {
        data: Box::new([0; RAMSIZE]),
        mapping: device::DeviceMapping {
            adrs: 0x10000000,
            size: RAMSIZE,
        },
    };
    println!("{:?}", ram);
}
```

 
