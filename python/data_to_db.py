import xlrd
import pandas as pd
import babelfish

from loguru import logger

from abc import ABC, abstractclassmethod
from collections import Counter
from guessit import guessit

import os
from tqdm import tqdm
import json
import sqlite3
from typing import List, Tuple, Dict, Any, Union

# -----------------
# 1. 核心解析函数
# -----------------


def str_process(name):
    return name.strip()


def parse_xls_to_data(
    xls_file_path: str, sheet_index: int = 0
) -> List[Tuple[str, str]]:
    """
    解析 XLS 文件，提取文件名和 JSON 描述信息。

    Args:
        xls_file_path: XLS 文件的完整路径。
        sheet_index: 要读取的工作表索引（默认为第一个工作表，即索引 0）。

    Returns:
        一个列表，其中每个元素是一个元组 (文件名, JSON字符串描述)。
    """
    data = []
    try:
        # 打开工作簿
        workbook = xlrd.open_workbook(xls_file_path)
        # 选中工作表
        sheet = workbook.sheet_by_index(sheet_index)

        # 遍历所有行，从第二行开始（假设第一行是标题）
        # 如果您的文件没有标题行，请将 range(1, ...) 改为 range(sheet.nrows)
        for row_idx in tqdm(range(2, sheet.nrows), desc="处理进度"):
            try:
                # 第一列：文件名称 (通常是字符串)
                movie_name_en = sheet.cell_value(row_idx, 0)
                # 第二列：文件描述 JSON 字符串
                movie_name_cn = sheet.cell_value(row_idx, 1)

                magnetlink = sheet.cell_value(row_idx, 2)

                # 将文件名转换为字符串，并去除可能的多余空格
                movie_name_cn = str(movie_name_cn).strip()
                # 将 JSON 字符串转换为字符串，并去除可能的多余空格
                movie_name_en = str_process(movie_name_en)
                magnetlink = str_process(magnetlink)
                data.append(
                    {
                        "movie_name_cn": movie_name_cn,
                        "movie_name_en": movie_name_en,
                        "magnetlink": magnetlink,
                    }
                )

            except IndexError:
                # 处理行中列数不足的情况
                print(f"Warning: Row {row_idx + 1} is incomplete and skipped.")
            except Exception as e:
                # 处理特定行数据读取错误
                print(f"Error processing row {row_idx + 1}: {e}")

    except xlrd.biffh.XLRDError:
        print(f"Error: Could not open or read the XLS file: {xls_file_path}")
    except Exception as e:
        print(f"An unexpected error occurred during XLS parsing: {e}")

    return data


def process_sequence_data(datas):
    info = []
    for data in datas:
        if isinstance(data, list):
            print(f"data = {data}")
            info.append("-".join(data))
        else:
            info.append(data)
    return info


# -----------------
# 2. SQLite 写入函数
# -----------------


def insert_data_to_sqlite(
    db_path: str,
    parsed_data: Union[str, List[Tuple[str, str]]],
    cache_file="movies.json",
):
    """
    将解析后的数据写入 SQLite 数据库。

    Args:
        db_path: SQLite 数据库文件的路径。
        parsed_data: 包含 (文件名, JSON字符串描述) 的列表。
    """
    # 假设 JSON 描述中包含 'author' 和 'size' 字段，我们提取它们。
    # 如果 JSON 结构不同，您需要修改 CREATE TABLE 语句和数据提取逻辑。

    conn = None
    db_name, _ = os.path.splitext(db_path)
    db_name = os.path.basename(db_name)
    sequence_data = []
    if isinstance(parsed_data, str):
        with open(parsed_data, "r") as f:
            parsed_data = json.load(f)
    try:
        # 连接到 SQLite 数据库 (如果文件不存在，会自动创建)
        with open(cache_file, "w") as f:
            json.dump(parsed_data, f)
        conn = sqlite3.connect(db_path)
        cursor = conn.cursor()

        # 1. 创建表格
        cursor.execute(
            f"CREATE TABLE IF NOT EXISTS {
                db_name
            } (id INTEGER PRIMARY KEY AUTOINCREMENT,title TEXT NOT NULL,alternative_title TEXT,year TEXT,screen_size TEXT,source TEXT,video_codec TEXT,video_profile TEXT,audio_codec TEXT,audio_profile TEXT,audio_channels TEXT,release_group TEXT,type TEXT,movie_name_cn TEXT,movie_name_en TEXT,magnetlink TEXT)"
        )

        # 2. 准备插入数据
        for idx, items in enumerate(parsed_data):
            len
            name_cn = ""
            name_en = ""
            magnetlink = ""

            # 提取 JSON 中的关键信息
            name_cn = items.get("movie_name_cn", "")
            name_en = items.get("movie_name_en", "")
            magnetlink = items.get("magnetlink", "")

            result = guessit(name_en)
            result.update(items)
            movie_keys = result.keys()
            values = list(result.values())
            values.insert(0, idx)
            sequence_data.append(tuple(values))

        insert_values = f"INSERT INTO {db_name} ({','.join(movie_keys)}) VALUES ({
            ','.join(['?' for i in range(len(movie_keys))])
        })"
        legal = tuple(process_sequence_data(i)
                      for i in sequence_data if len(i) == 15)

        for elem in legal:
            val = set([type(i) for i in elem])
            assert len(val) == 2, f"{
                len(val)} must be same with 2,but now is {elem}"
        cursor.executemany(insert_values, legal)

        # 4. 提交更改
        conn.commit()
        print(f"Successfully inserted {
              len(sequence_data)} records into {db_path}.")

    except sqlite3.Error as e:
        print(f"SQLite error occurred: {e}")
    except Exception as e:
        print(f"An unexpected error occurred during database operation: {e}")
    finally:
        if conn:
            conn.close()


# -----------------
# 3. 主执行函数
# -----------------


def process_xls_to_sqlite(xls_path: str, db_path: str):
    """
    执行整个流程：解析 XLS 文件并写入 SQLite 数据库。
    """
    print(f"--- 1. Starting to parse XLS file: {xls_path} ---")

    # 1. 解析 XLS 文件
    db_name, _ = os.path.splitext(db_path)
    db_name = os.path.basename(db_name)
    parsed_data = parse_xls_to_data(xls_path)

    if not parsed_data:
        print("No valid data found in the XLS file. Aborting.")
        return

    print(f"Successfully extracted {len(parsed_data)} records.")

    # 2. 写入 SQLite 数据库
    insert_data_to_sqlite(db_path, parsed_data)


class BasePipeline(ABC):
    def __init__(self, xls_file_path: str):
        self.xls_file_path = xls_file_path
        self.sheet_index = 0
        self.cache_file = "movies.json"

    def str_process(self, name):
        return name.strip()

    @abstractclassmethod
    def xls_to_dict(self, xls_file_path: str) -> List[Dict[str, str]]:
        pass

    @abstractclassmethod
    def process_info(self, info: Dict[str, str]):
        return info

    @abstractclassmethod
    def verify_info(self, info: Dict[str, str]):
        pass

    @abstractclassmethod
    def info_to_sql(self):
        pass

    def pipeline(self):
        logger.info(
            f"尝试提取xls文件:{self.xls_file_path}中的文件，提取从第 {self.sheet_index} 开始"
        )
        info = self.xls_to_dict(self.xls_file_path)
        logger.info(f"文件信息成功提取{len(info)}，当前初步验证文件信息是否正确")
        info = self.verify_info(info)
        logger.info(f"文件信息验证成功,尝试通过文件名称匹配更多信息，验证后信息条数为:{len(info)}")
        info = self.process_info(info)
        logger.info("文件信息补充成功,尝试写入信息到sqlite中")
        return info


class MagnetPipeline(BasePipeline):
    table_name = set(['country', 'mimetype', 'season', 'source', 'proper_count', 'subtitle_language', 'magnetlink', 'audio_profile', 'edition', 'date',
                      'episode_title', 'other', 'aspect_ratio', 'audio_channels', 'episode_details', 'language', 'cd', 'type', 'color_depth', 'screen_size', 'movie_name_cn', 'title', 'part', 'alternative_title', 'audio_codec', 'container', 'video_profile', 'movie_name_en', 'year',
                      'website', 'streaming_service', 'release_group', 'disc', 'video_codec', 'episode'])

    def __init__(self, xls_file: str, sheet_index: int, cache_file: str):
        super().__init__(xls_file)
        self.sheet_index = sheet_index
        self.cache_file = cache_file
        self.json_file = "sqlite.json"

    def xls_to_dict(self, xls_file_path: str) -> List[Dict[str, str]]:
        data = []
        try:
            workbook = xlrd.open_workbook(xls_file_path)
            # 选中工作表
            sheet = workbook.sheet_by_index(self.sheet_index)

            # 遍历所有行，从第二行开始（假设第一行是标题）
            # 如果您的文件没有标题行，请将 range(1, ...) 改为 range(sheet.nrows)
            for row_idx in tqdm(range(2, sheet.nrows), desc="处理进度"):
                try:
                    # 第一列：文件名称 (通常是字符串)
                    movie_name_en = sheet.cell_value(row_idx, 0)
                    # 第二列：文件描述 JSON 字符串
                    movie_name_cn = sheet.cell_value(row_idx, 1)

                    magnetlink = sheet.cell_value(row_idx, 2)

                    # 将文件名转换为字符串，并去除可能的多余空格
                    movie_name_cn = self.str_process(movie_name_cn)
                    # 将 JSON 字符串转换为字符串，并去除可能的多余空格
                    movie_name_en = self.str_process(movie_name_en)
                    magnetlink = str_process(magnetlink)
                    data.append(
                        {
                            "movie_name_cn": movie_name_cn,
                            "movie_name_en": movie_name_en,
                            "magnetlink": magnetlink,
                        }
                    )

                except IndexError:
                    # 处理行中列数不足的情况
                    logger.warning(f"Warning: Row {
                        row_idx + 1} is incomplete and skipped.")
                except Exception as e:
                    # 处理特定行数据读取错误
                    logger.warning(f"Error processing row {row_idx + 1}: {e}")

        except xlrd.biffh.XLRDError:
            print(f"Error: Could not open or read the XLS file: {
                  xls_file_path}")
        except Exception as e:
            print(f"An unexpected error occurred during XLS parsing: {e}")
        if self.cache_file and not os.path.exists(self.cache_file):
            with open(self.cache_file, "w") as f:
                json.dump(data, f)
        return data

    def process_info(self, info):
        res = []
        for movie_item in info:
            combine = {}
            result = guessit(movie_item['movie_name_en'])
            result.update(movie_item)
            for name in self.table_name:
                v = result.get(name, "")
                try:
                    if isinstance(v, babelfish.language.Language) or isinstance(v, babelfish.country.Country):
                        if hasattr(v, "alpha2"):
                            v = v.alpha2
                        if hasattr(v, "alpha3"):
                            v = v.alpha3
                except Exception as e:
                    logger.info(f"类型转换失败，错误信息为:{e} v = {v} type = {type(v)}")
            res.append(combine)
        types = set()
        if not os.path.exists(self.json_file):
            with open(self.json_file, 'r') as f:
                json.dump(res, f)
        return res

    def verify_info(self, info):
        """
        验证读取的xls文件信息是否正确
        """
        same_len = len(set([len(elem) for elem in info]))
        assert same_len == 1, f"info length must be same with 1,but now is {
            same_len}"
        return info

    def info_to_sql(self):
        """


        """
        pass


class FlashPipeline(BasePipeline):
    def __init__(self, xls_file: str, sheet_index: int, cache_file: str):
        super().__init__(xls_file)
        self.sheet_index = sheet_index
        self.cache_file = cache_file

    def xls_to_dict(self, xls_file):
        info = {}
        try:
            # 1. 使用 sheet_name=None 读取文件中的所有工作表
            # 返回的是一个字典：{'Sheet1': DataFrame, 'Sheet2': DataFrame, ...}
            xls_data = pd.read_excel(xls_file, sheet_name=None, skiprows=1)
        except FileNotFoundError:
            logger.error(f"错误：文件未找到。请检查路径是否正确: {file_path}")
            return {}
        except Exception as e:
            logger.error(f"读取 Excel 文件时发生错误: {e}")
            return {}

        # 2. 遍历所有工作表 (Sheet Name 是键，DataFrame 是值)
        for sheet_name, df in xls_data.items():
            logger.info(f"--- 正在处理工作表: {sheet_name} ---")

            # 假设第一列是电影名称，第二列是 JSON 字符串
            # 为了健壮性，这里假设列名分别是第一列和第二列的默认名 (0, 1)
            # 如果您的 Excel 文件有表头，请根据实际列名进行调整 (例如 'MovieName', 'MovieInfo')

            # 重命名列以便于引用，并确保列数正确
            if df.shape[1] < 2:
                logger.info(f"警告：工作表 {sheet_name} 列数不足，跳过。")
                continue

            df = df.rename(columns={0: 'movie_name', 1: 'movie_info'})

            # 3. 遍历每一行数据
            for index, row in df.iterrows():
                movie_name = row[0]
                movie_info = row[1]

                # 确保 movie_name 有效且 JSON 字符串存在
                if pd.isna(movie_name) or pd.isna(movie_info):
                    continue

                try:
                    # 4. 解析 JSON 字符串为 Python 字典
                    if movie_info.endswith(','):
                        movie_info = movie_info.strip(',')
                    info_dict = json.loads(movie_info)

                    # 5. 将结果存储到汇总字典中，以电影名称为键
                    if "files" not in info:
                        info['files'] = []
                    info["files"].append(info_dict)

                except json.JSONDecodeError as e:
                    # 处理 JSON 格式错误的行
                    logger.error(f"解析错误：工作表 {sheet_name} 中的电影 '{
                        movie_name}' JSON 格式无效。错误详情：{e}")
                except Exception as e:
                    logger.error(f"处理数据时发生意外错误：{e}")

        if self.cache_file and not os.path.exists(self.cache_file):
            with open(self.cache_file, 'w') as f:
                json.dump(info, f, ensure_ascii=False)
        return info

    def process_info(self, info):
        return info

    def verify_info(self, info):
        return info

    def info_to_sql(self):
        pass


# -----------------
# 4. 示例使用
# -----------------


if __name__ == "__main__":
    basepath = "/home/liushuai/文档/xwechat_files/bleedingfight_ae28/msg/file/2025-10"
    XLS_FILE = os.path.join(
        basepath, "5320部4K蓝光电影原盘磁力链接（115可直接观看） (3).xls"
    )
    flash_xls = os.path.join(basepath, "123云盘综合大包秒传文件链接在线文档.xlsx")
    SQLITE_DB = "movies.db"
    db_name = "movies"
    # process_xls_to_sqlite(XLS_FILE, SQLITE_DB)
    # pipe = MagnetPipeline(XLS_FILE, 0, 'movies.json')
    # pipe.pipeline()
    pipe = FlashPipeline(flash_xls, 0, 'flash_movies.json')
    pipe.pipeline()
