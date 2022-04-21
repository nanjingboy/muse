/// This file contains Unicode properties extracted from the ECMAScript
/// specification. The lists are extracted like so:
/// $$('#table-binary-unicode-properties > figure > table > tbody > tr >
/// td:nth-child(1) code').map(el => el.innerText)
use std::collections::HashMap;

use fancy_regex::Regex;
use lazy_static::lazy_static;

use crate::utils::get_regex_from_words;

// #table-unicode-general-category-values
const UNICODE_GENERAL_CATEGORY_VALUES: &str = "Cased_Letter|LC|Close_Punctuation|Pe|Connector_Punctuation|Pc|Control|Cc|cntrl|Currency_Symbol|Sc|Dash_Punctuation|Pd|Decimal_Number|Nd|digit|Enclosing_Mark|Me|Final_Punctuation|Pf|Format|Cf|Initial_Punctuation|Pi|Letter|L|Letter_Number|Nl|Line_Separator|Zl|Lowercase_Letter|Ll|Mark|M|Combining_Mark|Math_Symbol|Sm|Modifier_Letter|Lm|Modifier_Symbol|Sk|Nonspacing_Mark|Mn|Number|N|Open_Punctuation|Ps|Other|C|Other_Letter|Lo|Other_Number|No|Other_Punctuation|Po|Other_Symbol|So|Paragraph_Separator|Zp|Private_Use|Co|Punctuation|P|punct|Separator|Z|Space_Separator|Zs|Spacing_Mark|Mc|Surrogate|Cs|Symbol|S|Titlecase_Letter|Lt|Unassigned|Cn|Uppercase_Letter|Lu";

lazy_static! {
    // #table-binary-unicode-properties
    static ref UNICODE_BINARY_PROPERTIES: HashMap<i32, String> = {
        let ecma_9_binary_properties = "ASCII|ASCII_Hex_Digit|AHex|Alphabetic|Alpha|Any|Assigned|Bidi_Control|Bidi_C|Bidi_Mirrored|Bidi_M|Case_Ignorable|CI|Cased|Changes_When_Casefolded|CWCF|Changes_When_Casemapped|CWCM|Changes_When_Lowercased|CWL|Changes_When_NFKC_Casefolded|CWKCF|Changes_When_Titlecased|CWT|Changes_When_Uppercased|CWU|Dash|Default_Ignorable_Code_Point|DI|Deprecated|Dep|Diacritic|Dia|Emoji|Emoji_Component|Emoji_Modifier|Emoji_Modifier_Base|Emoji_Presentation|Extender|Ext|Grapheme_Base|Gr_Base|Grapheme_Extend|Gr_Ext|Hex_Digit|Hex|IDS_Binary_Operator|IDSB|IDS_Trinary_Operator|IDST|ID_Continue|IDC|ID_Start|IDS|Ideographic|Ideo|Join_Control|Join_C|Logical_Order_Exception|LOE|Lowercase|Lower|Math|Noncharacter_Code_Point|NChar|Pattern_Syntax|Pat_Syn|Pattern_White_Space|Pat_WS|Quotation_Mark|QMark|Radical|Regional_Indicator|RI|Sentence_Terminal|STerm|Soft_Dotted|SD|Terminal_Punctuation|Term|Unified_Ideograph|UIdeo|Uppercase|Upper|Variation_Selector|VS|White_Space|space|XID_Continue|XIDC|XID_Start|XIDS".to_owned();
        let ecma_10_binary_properties = format!("{:}|Extended_Pictographic", ecma_9_binary_properties);
        let ecma_11_binary_properties = ecma_10_binary_properties.clone();
        let ecma_12_binary_properties = format!("{:}|EBase|EComp|EMod|EPres|ExtPict", ecma_11_binary_properties);
        let ecma_13_binary_properties = ecma_12_binary_properties.clone();
        let mut result: HashMap<i32, String> = HashMap::with_capacity(5);
        result.insert(9, ecma_9_binary_properties);
        result.insert(10, ecma_10_binary_properties);
        result.insert(11, ecma_11_binary_properties);
        result.insert(12, ecma_12_binary_properties);
        result.insert(13, ecma_13_binary_properties);
        result
    };

    // #table-unicode-script-values
    static ref UNICODE_SCRIPT_VALUES: HashMap<i32, String> = {
        let ecma_9_script_values = "Adlam|Adlm|Ahom|Anatolian_Hieroglyphs|Hluw|Arabic|Arab|Armenian|Armn|Avestan|Avst|Balinese|Bali|Bamum|Bamu|Bassa_Vah|Bass|Batak|Batk|Bengali|Beng|Bhaiksuki|Bhks|Bopomofo|Bopo|Brahmi|Brah|Braille|Brai|Buginese|Bugi|Buhid|Buhd|Canadian_Aboriginal|Cans|Carian|Cari|Caucasian_Albanian|Aghb|Chakma|Cakm|Cham|Cham|Cherokee|Cher|Common|Zyyy|Coptic|Copt|Qaac|Cuneiform|Xsux|Cypriot|Cprt|Cyrillic|Cyrl|Deseret|Dsrt|Devanagari|Deva|Duployan|Dupl|Egyptian_Hieroglyphs|Egyp|Elbasan|Elba|Ethiopic|Ethi|Georgian|Geor|Glagolitic|Glag|Gothic|Goth|Grantha|Gran|Greek|Grek|Gujarati|Gujr|Gurmukhi|Guru|Han|Hani|Hangul|Hang|Hanunoo|Hano|Hatran|Hatr|Hebrew|Hebr|Hiragana|Hira|Imperial_Aramaic|Armi|Inherited|Zinh|Qaai|Inscriptional_Pahlavi|Phli|Inscriptional_Parthian|Prti|Javanese|Java|Kaithi|Kthi|Kannada|Knda|Katakana|Kana|Kayah_Li|Kali|Kharoshthi|Khar|Khmer|Khmr|Khojki|Khoj|Khudawadi|Sind|Lao|Laoo|Latin|Latn|Lepcha|Lepc|Limbu|Limb|Linear_A|Lina|Linear_B|Linb|Lisu|Lisu|Lycian|Lyci|Lydian|Lydi|Mahajani|Mahj|Malayalam|Mlym|Mandaic|Mand|Manichaean|Mani|Marchen|Marc|Masaram_Gondi|Gonm|Meetei_Mayek|Mtei|Mende_Kikakui|Mend|Meroitic_Cursive|Merc|Meroitic_Hieroglyphs|Mero|Miao|Plrd|Modi|Mongolian|Mong|Mro|Mroo|Multani|Mult|Myanmar|Mymr|Nabataean|Nbat|New_Tai_Lue|Talu|Newa|Newa|Nko|Nkoo|Nushu|Nshu|Ogham|Ogam|Ol_Chiki|Olck|Old_Hungarian|Hung|Old_Italic|Ital|Old_North_Arabian|Narb|Old_Permic|Perm|Old_Persian|Xpeo|Old_South_Arabian|Sarb|Old_Turkic|Orkh|Oriya|Orya|Osage|Osge|Osmanya|Osma|Pahawh_Hmong|Hmng|Palmyrene|Palm|Pau_Cin_Hau|Pauc|Phags_Pa|Phag|Phoenician|Phnx|Psalter_Pahlavi|Phlp|Rejang|Rjng|Runic|Runr|Samaritan|Samr|Saurashtra|Saur|Sharada|Shrd|Shavian|Shaw|Siddham|Sidd|SignWriting|Sgnw|Sinhala|Sinh|Sora_Sompeng|Sora|Soyombo|Soyo|Sundanese|Sund|Syloti_Nagri|Sylo|Syriac|Syrc|Tagalog|Tglg|Tagbanwa|Tagb|Tai_Le|Tale|Tai_Tham|Lana|Tai_Viet|Tavt|Takri|Takr|Tamil|Taml|Tangut|Tang|Telugu|Telu|Thaana|Thaa|Thai|Thai|Tibetan|Tibt|Tifinagh|Tfng|Tirhuta|Tirh|Ugaritic|Ugar|Vai|Vaii|Warang_Citi|Wara|Yi|Yiii|Zanabazar_Square|Zanb".to_owned();
        let ecma_10_script_values = format!("{:}|Dogra|Dogr|Gunjala_Gondi|Gong|Hanifi_Rohingya|Rohg|Makasar|Maka|Medefaidrin|Medf|Old_Sogdian|Sogo|Sogdian|Sogd", ecma_9_script_values);
        let ecma_11_script_values = format!("{:}|Elymaic|Elym|Nandinagari|Nand|Nyiakeng_Puachue_Hmong|Hmnp|Wancho|Wcho", ecma_10_script_values);
        let ecma_12_script_values = format!("{:}|Chorasmian|Chrs|Diak|Dives_Akuru|Khitan_Small_Script|Kits|Yezi|Yezidi", ecma_11_script_values);
        let ecma_13_script_values = format!("{:}|Cypro_Minoan|Cpmn|Old_Uyghur|Ougr|Tangsa|Tnsa|Toto|Vithkuqi|Vith", ecma_12_script_values);
        let mut result: HashMap<i32, String> = HashMap::with_capacity(5);
        result.insert(9, ecma_9_script_values);
        result.insert(10, ecma_10_script_values);
        result.insert(11, ecma_11_script_values);
        result.insert(12, ecma_12_script_values);
        result.insert(13, ecma_13_script_values);
        result
    };
}

#[derive(Debug, Clone)]
pub struct NonBinary {
    pub general_category: Regex,
    pub script: Regex,
    pub script_extensions: Regex,
    pub gc: Regex,
    pub sc: Regex,
    pub scx: Regex,
}

#[derive(Debug, Clone)]
pub struct UnicodeProperties {
    pub binary: Regex,
    pub non_binary: NonBinary,
}

impl UnicodeProperties {
    fn new(ecma_version: i32) -> Self {
        let binary = get_regex_from_words(&format!(
            "{:}|{:}",
            UNICODE_BINARY_PROPERTIES.get(&ecma_version).unwrap(),
            UNICODE_GENERAL_CATEGORY_VALUES
        ));
        let general_category = get_regex_from_words(UNICODE_GENERAL_CATEGORY_VALUES);
        let script = get_regex_from_words(UNICODE_SCRIPT_VALUES.get(&ecma_version).unwrap());
        UnicodeProperties {
            binary,
            non_binary: NonBinary {
                general_category: general_category.clone(),
                script: script.clone(),
                script_extensions: script.clone(),
                gc: general_category.clone(),
                sc: script.clone(),
                scx: script.clone(),
            },
        }
    }
}

lazy_static! {
    static ref UNICODE_PROPERTY_VALUES: HashMap<i32, UnicodeProperties> = {
        let mut result: HashMap<i32, UnicodeProperties> = HashMap::with_capacity(5);
        result.insert(9, UnicodeProperties::new(9));
        result.insert(10, UnicodeProperties::new(10));
        result.insert(11, UnicodeProperties::new(11));
        result.insert(12, UnicodeProperties::new(12));
        result.insert(13, UnicodeProperties::new(13));
        result
    };
}

pub fn get_unicode_properties(ecma_version: i32) -> Option<&'static UnicodeProperties> {
    UNICODE_PROPERTY_VALUES.get(&ecma_version)
}
