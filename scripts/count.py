from os import path
import matplotlib.pyplot as plt

LLVM_TRUNK_VERSION = 17

LLVM_VERSION_TO_DATE = {
    # 18: "2024/05/05",
    17: "2023/09/09",
    16: "2023/05/17",
    15: "2022/09/06",
    14: "2022/05/25",
    13: "2021/10/04",
    12: "2021/04/14",
    11: "2020/10/12",
    10: "2020/05/24",
    9:  "2019/09/19",
    8:  "2019/05/20",
    7:  "2018/09/19",
    6:  "2018/05/08",
    5:  "2017/09/07",
    4: "2017/05/13",
    3: "2011/12/01",
}

ERROR_INTRO_DATE = [
    "2020/07/18",
    "2008/11/17",
    "2008/11/17",
    "2008/11/03",
    "2008/11/17",
    "2014/10/20",
    "2016/01/08",
    "2017/02/17",
    "2017/04/11",
    "2017/04/11",
    "2022/07/08",
    "2015/11/03",
    "2015/04/14",
    "2015/04/14",
    "2017/09/09",
    "2017/09/09",
    "2020/03/24",
    "2020/03/24",
    "2019/07/31",
    "2020/01/15",
    "2015/08/14",
    "2008/12/03",
    "2017/04/23",
    "2020/11/07",
    "2020/12/30",
    "2019/10/14",
    "2009/12/22",
    "2008/08/26",
    "2008/08/26",
    "2017/11/17",
    "2017/11/17",
    "2017/11/17",
    "2020/10/07",
    "2017/11/17",
    "2023/05/11",
    "2017/04/27",
    "2018/12/04",
    "2017/04/27",
    "2017/04/27",
    "2017/05/25",
    "2016/07/15",
    "2018/08/03",
    "2015/11/03",
    "2020/06/04",
    "2020/06/04",
    # "2009/12/31", # Reassociate-L851
    # "2015/05/15", # SpeculativeExecution-L331
]

if __name__ == "__main__":
    count_map = { key: [] for key in LLVM_VERSION_TO_DATE.keys()}
    for date in ERROR_INTRO_DATE:
        for k, v in LLVM_VERSION_TO_DATE.items():
            if date < v:
                count_map[k].append(date)

    x = list(count_map.keys())
    scaled_x = list(map(lambda i: i * 2, x))
    y = list(map(lambda x: len(x), count_map.values()))

    plt.figure(figsize=(13, 4.3))
    plt.tight_layout()

    for i, v in zip(scaled_x, y):
        plt.text(i, v + 0.5, str(v), ha='center')

    for v in [10, 20, 30, 40]:
        plt.axhline(y=v, color='#000000', linestyle='--', linewidth=0.5)

    plt.bar(scaled_x, y, color='grey')
    plt.xlabel("LLVM Versions", fontsize=12)
    plt.ylabel("Number of Fixes", fontsize=12)
    plt.xticks(scaled_x, list(map(lambda x: str(x), x)), fontsize=12)
    plt.yticks(fontsize=12)
    plt.savefig(f"{path.dirname(__file__)}/affected-LLVM-version.pdf")

    plt.show()

