from typing import List
import base62
from pathlib import Path
from typing import Any, Union, Dict, List, Tuple
from tqdm import tqdm
import sqlite3
from typing import Any, Dict, List
import os
import pandas as pd
import json
from abc import ABC, abstractmethod
from typing import Any, Dict, List, Union
from loguru import logger

IntermediateData = List[Dict[str, Any]]


ebook = {".epub", ".mobi"}
audio_type = {
    ".mp3",
    ".flac",
    ".wav",
    ".wma",
    ".aac",
    ".aiff",
    ".aif",
    ".dts",
    ".m4a",
    ".ra",
    ".ape",
    ".tta",
    ".wv",
    ".dsf",
    ".mlp",
    ".mka",
    ".mp3",
    ".wma",
    ".wav",
    ".flac",
}
video_type = {
    ".mkv",
    ".mp4",
    ".avi",
    ".rmvb",
    ".rm",
    ".flv",
    ".ts",
    ".vob",
    ".wmv",
    ".mpg",
    ".mpeg",
    ".mov",
    ".m4v",
    ".f4v",
    ".m2ts",
    ".tp",
    ".ts",
    ".vob",
    ".flv",
    ".avi",
    ".rm",
    ".rmvb",
    ".wmv",
    ".mpg",
    ".mkv",
}
picture_type = {
    ".jpeg",
    ".png",
    ".gif",
    ".bmp",
    ".svg",
    ".tif",
    ".tiff",
    ".ico",
    ".icns",
    ".webp",
    ".psd",
    ".jpg",
    ".jpeg",
    ".png",
    ".gif",
    ".bmp",
    ".ico",
    ".tif",
    ".jpg",
}
zip_type = {".zip", ".rar", ".7z", ".gz", ".cab", ".001", ".002", ".003"}
srt_type = {
    ".srt",
    ".ass",
    ".ssa",
    ".vtt",
    ".smi",
    ".sub",
    ".sup",
    ".lrc",
    ".ksc",
    ".srt",
    ".ass",
    ".ssa",
}
cd_type = {
    ".iso",
    ".nrg",
    ".mdf",
    ".mds",
    ".cue",
    ".bin",
    ".bdmv",
    ".bdjo",
    ".clpi",
    ".mpls",
    ".mds",
    ".iso",
    ".cue",
    ".dat",
}
play_type = {".m3u8", ".wpl", ".m3u", ".fpl", ".m3u"}
text_type = {
    ".txt",
    ".pdf",
    ".doc",
    ".docx",
    ".xls",
    ".xlsx",
    ".rtf",
    ".chm",
    ".nfo",
    ".log",
    ".plist",
    ".json",
    ".txt",
    ".pdf",
    ".log",
}
font_type = {".ttf", ".otf", ".ttc", ".ttc", ".otf"}
exec_type = {
    ".exe",
    ".jar",
    ".apk",
    ".cmd",
    ".bat",
    ".inf",
    ".ini",
    ".cfg",
    ".tmp",
    ".downloading",
    ".backup",
    ".bad",
    ".ds_store",
    ".lnk",
    ".url",
    ".qrc",
    ".stackdump",
    ".app",
    ".dat",
    ".meta",
    ".rsrc",
    ".sfk",
    ".sfv",
    ".ffp",
    ".md5",
    ".pk",
    ".asf",
    ".kux",
    ".nfo",
    ".torrent",
    ".tobedelete",
    ".alist_to_delete",
    ".yunpantmp",
    ".!ut",
    ".html",
    ".htm",
    ".mht",
    ".css",
    ".xml",
    ".conf",
    ".xltd",
    ".qmfpd2",
    ".crt",
    ".jasfv",
    ".rpg",
}
nintendo_type = {".xci", ".xcz", ".nsz", ".nsp"}

all_file_types = {
    "ebook": ebook,
    "audio": audio_type,
    "video": video_type,
    "picture": picture_type,
    "zip": zip_type,
    "srt": srt_type,
    "cd": cd_type,
    "play": play_type,
    "text": text_type,
    "font": font_type,
    "exec": exec_type,
    "nintendo": nintendo_type,
}


def find_files(root_dir: str, ext=".json") -> List[Path]:
    """
    递归查找指定目录下所有的 .json 文件。

    Args:
        root_dir: 开始搜索的根目录路径（字符串）。

    Returns:
        一个包含所有找到的 JSON 文件 Path 对象的列表。
    """

    # 检查目录是否存在
    root_path = Path(root_dir)
    if not root_path.is_dir():
        print(f"错误: 目录 '{root_dir}' 不存在或不是一个目录。")
        return []

    json_files = []

    # os.walk 会生成一个三元组 (当前目录路径, 子目录列表, 文件列表)，并递归遍历
    for dirpath, dirnames, filenames in os.walk(root_path):
        # dirpath 是当前正在遍历的目录的路径

        for filename in filenames:
            # 检查文件是否以 .json 结尾，并且不区分大小写
            if filename.lower().endswith(ext):
                # 构造完整的 JSON 文件路径并添加到列表中
                full_path = Path(dirpath) / filename
                json_files.append(full_path)

    return json_files


def find_files_key_recursively(data: Union[Dict, List, Any], keys: str) -> bool:
    """
    递归查找嵌套的字典/列表结构中是否存在键为 'files' 的键。

    参数:
        data: 待搜索的嵌套对象 (字典、列表、或任何其他类型)。

    返回:
        如果找到 'files' 键，返回 True；否则返回 False。
    """

    # 1. 基础情况：处理字典
    if isinstance(data, dict):
        # 检查当前字典是否包含目标键
        if keys in data:
            return True

        # 递归遍历字典的值
        for value in data.values():
            if find_files_key_recursively(value):
                return True

    # 2. 基础情况：处理列表或元组
    elif isinstance(data, (list, tuple)):
        # 递归遍历列表/元组的元素
        for item in data:
            if find_files_key_recursively(item):
                return True

    # 3. 基础情况：其他类型 (如字符串、整数等) 不再深入，直接返回 False
    return False


class BaseConverter(ABC):
    """
    所有转换器的抽象基类。
    """

    @property
    @abstractmethod
    def key(self) -> str:
        """源文件格式（如 'xls', 'json'）。"""
        pass

    @abstractmethod
    def read(self, input_path: str) -> IntermediateData:
        """从源文件路径读取数据，并转换为中间格式。"""
        pass

    @abstractmethod
    def verify(self, input_path: Union[Dict, Tuple, List]) -> bool:
        """从源文件路径读取数据，并转换为中间格式。"""
        return True

    @abstractmethod
    def write(self, data: IntermediateData, output_path: str) -> None:
        """将中间格式数据写入目标文件路径。"""
        pass

    def convert(self, input_path: str) -> None:
        """执行完整的转换流程。"""
        logger.info(f"尝试转换{input_path}")
        data = self.read(input_path)
        logger.info(f"从文件 {input_path} 读取 {len(data)} 条数据 ")
        return data


class XlsxToDictConverter(BaseConverter):
    key = "xlsx"

    def read(self, input_path: str) -> IntermediateData:
        """读取 XLS 文件并转换为 IntermediateData (列表的字典)。"""
        logger.info(f"  -> 读取 XLS/Excel 文件: {input_path}")
        # 假设我们只读取第一个工作表
        df = pd.read_excel(input_path, engine="openpyxl")
        # 将 DataFrame 转换为列表的字典
        return df.to_dict("records")

    def write(self, data: IntermediateData, output_path: str) -> None:
        """将 IntermediateData 写入 JSON 文件。"""
        logger.info(f"  -> 写入 JSON 文件: {output_path}")
        with open(output_path, "w", encoding="utf-8") as f:
            json.dump(data, f, indent=4, ensure_ascii=False)

    def verify(self, data) -> bool:
        return True


class JsonToDictConverter(BaseConverter):
    key = "json"

    def read(self, input_path: str) -> IntermediateData:
        """读取 JSON 文件并转换为 IntermediateData。"""
        logger.info(f"  -> 尝试读取 JSON 文件: {input_path}")
        try:
            with open(input_path, "r", encoding="utf-8") as f:
                data = json.load(f)
        except Exception as e:
            logger.error(f"读取文件:{input_path} 出现异常:{e}")

        return data

    def write(self, data: IntermediateData, output_path: str) -> None:
        """将 IntermediateData 写入JSON 文件。"""
        logger.info(f"  -> 写入 JSON 文件: {output_path}")
        df = pd.DataFrame(data)
        writer = pd.ExcelWriter(output_path, engine="openpyxl")
        df.to_excel(writer, index=False, sheet_name="Sheet1")
        writer.close()

    def verify(self, data):
        return find_files_key_recursively(data)

    def __repr__(self):
        return "Json to Dict Converter"


class ConverterManager:
    """
    转换器调度器，负责注册、查找和执行转换任务。
    """

    def __init__(self):
        # 存储已注册的转换器，键为 (source_format, target_format)
        self._converters: Dict[str, BaseConverter] = {}
        self.register_converter(XlsxToDictConverter())
        self.register_converter(JsonToDictConverter())
        # 您可以继续注册其他转换器，例如 CsvToJsonConverter()

    def register_converter(self, converter: BaseConverter) -> None:
        """注册一个具体的转换器实例。"""
        if converter.key in self._converters:
            logger.info(f"警告：转换器 {converter.key} 已被覆盖。")
        self._converters[converter.key] = converter
        logger.info(f"注册转换器:{converter.key}")

    def __repr__(self):
        return f"ConverterManager[{len(self._converters)}]"

    def get_converter(self, key: str) -> BaseConverter:
        """根据源格式和目标格式获取对应的转换器。"""
        converter = self._converters.get(key)
        if not converter:
            raise ValueError(f"找不到从 {converter}!")
        return converter

    def verify(self, data):
        pass

    def convert(self, input_path: str) -> Dict:
        """
        根据输入和输出文件的扩展名自动选择转换器并执行转换。
        """
        source_ext = os.path.splitext(input_path)[1].lstrip(".").lower()

        if not source_ext:
            raise ValueError("输入或输出文件路径必须包含有效的扩展名。")

        converter = self.get_converter(source_ext)
        data = converter.convert(input_path)
        logger.info(f"写入{input_path} {len(data)} 条数据")
        return data


def filter_list(infos, file_type=["mkv"]):
    for file in infos:
        pass


def filter_and_merge(data: Dict, merge: Dict, reformat=True):
    """
    data:最终的Dict
    merge：原始的Dict
    """
    assert find_files_key_recursively(merge, "files"), (
        f"{merge.keys()} must contain files"
    )
    files = merge["files"]
    assert isinstance(files, list), f"type(files) must be list,but now = {type(files)}"
    all_files = set()
    type_file = "video"
    for file_info in files:
        file_name = file_info["path"]
        name, ext = os.path.splitext(file_name)
        all_files.add(ext)
        ext = ext.lower()
        # 遍历文件类型
        for tp, tp_value in all_file_types.items():
            if ext in tp_value:
                type_file = tp

        if type_file in data:
            if reformat and len(file_info["etag"]) != 32:
                file_info["etag"] = (
                    base62.decode(file_info["etag"], charset=base62.CHARSET_INVERTED)
                    .to_bytes(16)
                    .hex()
                )
            data[type_file].append(file_info)
        else:
            logger.warning(
                f"{ext} 不属于 {all_file_types.keys()} 中任何一种类型，不会统计！ {
                    file_info
                } 将被丢弃"
            )
    return data, all_files


def parse_and_save_json(json_file_path, db_base_dir="."):
    """
    解析 JSON 文件，并根据顶级键将数据写入独立的 SQLite 数据库文件。

    :param json_file_path: JSON 文件路径
    :param db_base_dir: 存放生成的 SQLite 数据库文件的目录
    """

    # 确保数据库存放目录存在
    os.makedirs(db_base_dir, exist_ok=True)

    try:
        with open(json_file_path, "r", encoding="utf-8") as f:
            data = json.load(f)
    except FileNotFoundError:
        print(f"错误：未找到文件 {json_file_path}")
        return
    except json.JSONDecodeError:
        print(f"错误：JSON 文件 {json_file_path} 格式不正确")
        return

    # 遍历 JSON 中的每一个顶级键（即不同的数据类别，如 'video', 'audio'）
    for key, file_list in data.items():
        # 将键名作为数据库文件名（例如：video.db, audio.db）
        db_file_name = os.path.join(db_base_dir, f"{key}.db")
        table_name = key  # 使用键名作为表名

        print(f"\n--- 正在处理键: {key}，写入数据库: {db_file_name} ---")

        # 连接到 SQLite 数据库（如果文件不存在则会自动创建）
        conn = None
        try:
            conn = sqlite3.connect(db_file_name)
            cursor = conn.cursor()

            # 1. 创建表格
            # 使用 IF NOT EXISTS 避免重复创建时报错
            create_table_sql = f"""
            CREATE TABLE IF NOT EXISTS {table_name} (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                path TEXT NOT NULL,
                size TEXT,
                etag TEXT,
                UNIQUE(path, etag)
            );
            """
            cursor.execute(create_table_sql)
            print(f"表 {table_name} 已创建或已存在。")

            # 2. 插入数据
            # insert_sql = f"INSERT INTO {table_name} (path, size, etag) VALUES (?, ?, ?)"
            insert_sql = f"INSERT OR REPLACE INTO {
                table_name
            } (path, size, etag) VALUES (?, ?, ?)"

            # 使用批量插入以提高性能
            data_to_insert = []

            if isinstance(file_list, list):
                for item in file_list:
                    # 确保字典中包含所需的键
                    if all(k in item for k in ["path", "size", "etag"]):
                        data_to_insert.append(
                            (item["path"], item["size"], item["etag"])
                        )
                    else:
                        print(f"警告：跳过格式不正确的条目: {item}")
            else:
                print(f"警告：键 '{key}' 对应的值不是列表，跳过。")
                continue

            # 执行批量插入
            if data_to_insert:
                cursor.executemany(insert_sql, data_to_insert)
                conn.commit()
                print(f"成功插入 {len(data_to_insert)} 条记录到 {db_file_name}")
            else:
                print("列表为空或没有有效数据可插入。")

        except sqlite3.Error as e:
            print(f"SQLite 数据库操作错误: {e}")
        finally:
            if conn:
                conn.close()


# # --- 示例用法 ---

# # 1. 准备一个示例 JSON 文件
# example_json_content = {
#     "video": [
#         {'path': '电影/科幻/Interstellar.mp4', 'size': '10.5GB', 'etag': 'Vd89sL23A'},
#         {'path': '剧集/美剧/GOT_S01E01.mkv', 'size': '2.1GB', 'etag': 'Xy2aB7c8F'}
#     ],
#     "audio": [
#         {'path': '音乐库/无损音乐/005.WAV系列/音乐3/雨果系列290张-6/雨果唱片-《广东小曲》/新建 文本文档.txt', 'size': '786', 'etag': '5Z4EKevrwtDlrB8wJgEVas'},
#         {'path': '音乐库/古典乐/Beethoven_No5.flac', 'size': '50MB', 'etag': '9JkL4oPrs'}
#     ],
#     "image": [
#         {'path': '图片/风景/sunset.jpg', 'size': '2MB', 'etag': 'T7uQzW1xY'}
#     ]
# }

# # 写入临时的 JSON 文件
# temp_json_path = "file_metadata.json"
# with open(temp_json_path, 'w', encoding='utf-8') as f:
#     json.dump(example_json_content, f, indent=4)

# 2. 调用函数执行解析和存储
# 数据库文件将被创建在当前目录下的 'dbs' 文件夹中

# 3. 清理（可选：删除临时文件和生成的数据库目录）
# os.remove(temp_json_path)
# os.rmdir("dbs")

if __name__ == "__main__":
    if not os.path.exists("test_in.xlsx"):
        df = pd.DataFrame([{"Name": "张三", "Age": 30}, {"Name": "李四", "Age": 25}])
        df.to_excel("test_in.xlsx", index=False)

    if not os.path.exists("test_in.json"):
        with open("test_in.json", "w") as f:
            json.dump([{"Key": "Value1"}, {"Key": "Value2"}], f)

    manager = ConverterManager()

    json_file = "/home/liushuai/下载/123/"
    files = find_files(json_file)
    # all_json = {"files": []}
    all_json = {t: [] for t in all_file_types.keys()}

    all_types = set()
    for file in tqdm(files):
        try:
            name, ext = os.path.splitext(file)
            name = os.path.basename(name)
            logger.info(f"尝试转换{name}文件!")
            data = manager.convert(file)
            all_json, files_set = filter_and_merge(all_json, data)
            [all_types.add(e) for e in list(files_set)]
        except Exception as e:
            logger.info(f"转换失败: {e}\n")
        finally:
            with open("/tmp/file.list", "w") as f:
                for file in all_types:
                    f.write(f"{file}\n")

    saved_json = "merge_all.json"
    with open(saved_json, "w") as f:
        json.dump(all_json, f, ensure_ascii=False, indent=4)

    parse_and_save_json(saved_json, db_base_dir="dbs")
