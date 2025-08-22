# Honor Code

关于在 Rust 项目中导入另一个文件中的常量,我向 deepseek 提问了 in the same cargo project but another rust source file,i have define some vectors:pub const FINAL: &[&str] = &["..."], how can i import or include it in main.rs? ，deepseek回答了

    在 Rust 中导入其他文件的常量：

    创建文件：如 constants.rs

    声明模块：在 main.rs 中使用 mod constants;

    导入常量：使用 use constants::FINAL;

    使用常量：直接使用 FINAL
启发了我在 mod builtin_words; 中的实现。

关于将 String 或 &str 转换为 Vec<char> 或 &[char],我向 deepseek 提问了 can i transform String or &str into vec:&[char]，deepseek回答了 

    方法一：转换为 Vec<char>（拥有所有权的版本）
    从 String 转换：
    let s = String::from("你好 Rust!");
    let char_vec: Vec<char> = s.chars().collect();
，启发了我在let mut guess_word_vector:Vec<char> = guess.chars().collect()
中的实现。


关于Rust 中字符索引计算和数组操作,我向 deepseek 提问了'G' => chracter_status[guess_word_vector[i]-'A']='G',，deepseek回答了 

    Rust 不能直接像 C++ 那样用字符进行算术运算。以下是正确的实现方法：
    在 C++ 中你可以这样写：character_status[guess_word_vector[i] - 'A'] = 'G';
    但在 Rust 中，字符不能直接进行算术运算，需要先转换为数字。
    方法一：使用 as u8 转换
启发了我在'G' => chracter_status[((guess_word_vector[i] as u8)-b'a' )as usize]='G',
中的实现。

关于判断输入字符串是否在常量数组中,我向 deepseek 提问了how  to judge if the input &str is in the pub const ACCEPTABLE: &[&str]?，deepseek回答了 contains，启发了我在 if !(ACCEPTABLE.contains(&guess.trim())) 中的实现。

关于大小写转换,我向 deepseek 提问了upper,lower transform，deepseek回答启发了我在guess = guess.to_lowercase();中的实现。


