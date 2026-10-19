"""Reviewed Chengdu player portraits, not a live caption/order matching rule.

Source: https://www.sohu.com/picture/991014558 (2026-02-28).
The gallery also includes coaches and unnamed staff; only named player photos are used.
Aliases reviewed: 董岩鋒/董岩峰, 王子铭/王子鸣, 席尔瓦/韦林顿-席尔瓦.
Versioned object names preserve old files and invalidate client image caches.
"""

PLAYER_AVATAR_OVERRIDES: dict[tuple[int, int], tuple[str, str]] = {
    (77680, 113495): (  # 刘殿座 — gallery caption: 刘殿座
        "https://q4.itc.cn/q_85,c_zoom,w_600/images03/20260228/d7e69dfd2e014d38b2ef8945f7f565fe.jpeg",
        "113495-chengdu-20260228.jpg",
    ),
    (77680, 156047): (  # 冯卓毅 — gallery caption: 冯卓毅
        "https://q4.itc.cn/q_85,c_zoom,w_600/images03/20260228/7f1024aac3bb41709cc6ae3dafbbf5f1.jpeg",
        "156047-chengdu-20260228.jpg",
    ),
    (77680, 156225): (  # 廖力生 — gallery caption: 廖力生
        "https://q7.itc.cn/q_85,c_zoom,w_600/images03/20260228/8823a2a979254a78a5e5815a669c5b3d.jpeg",
        "156225-chengdu-20260228.jpg",
    ),
    (77680, 166406): (  # 杨明洋 — gallery caption: 杨明洋
        "https://q1.itc.cn/q_85,c_zoom,w_600/images03/20260228/cf0f9c6351b5449c832d3f9719acba6e.jpeg",
        "166406-chengdu-20260228.jpg",
    ),
    (77680, 181302): (  # 甘超 — gallery caption: 甘超
        "https://q1.itc.cn/q_85,c_zoom,w_600/images03/20260228/e13648628feb4f0082edb498f4d8bca8.jpeg",
        "181302-chengdu-20260228.jpg",
    ),
    (77680, 184911): (  # 韦世豪 — gallery caption: 韦世豪
        "https://q3.itc.cn/q_85,c_zoom,w_600/images03/20260228/800b9c6aa26f4ca2a293ecacff9c81f5.jpeg",
        "184911-chengdu-20260228.jpg",
    ),
    (77680, 198252): (  # 罗慕洛 — gallery caption: 罗慕洛
        "https://q5.itc.cn/q_85,c_zoom,w_600/images03/20260228/047e7f4c79344f118dba522761a21364.jpeg",
        "198252-chengdu-20260228.jpg",
    ),
    (77680, 204211): (  # 费利佩 — gallery caption: 费利佩
        "https://q5.itc.cn/q_85,c_zoom,w_600/images03/20260228/2b4dc1e566724cebaa3cc8948b0a87a1.jpeg",
        "204211-chengdu-20260228.jpg",
    ),
    (77680, 223673): (  # 韩鹏飞 — gallery caption: 韩鹏飞
        "https://q4.itc.cn/q_85,c_zoom,w_600/images03/20260228/16f8c58f9636433b87eee93c14a2ad82.jpeg",
        "223673-chengdu-20260228.jpg",
    ),
    (77680, 422194): (  # 蹇韬 — gallery caption: 蹇韬
        "https://q8.itc.cn/q_85,c_zoom,w_600/images03/20260228/0ace446c89524a33bcb25eb2ef618312.jpeg",
        "422194-chengdu-20260228.jpg",
    ),
    (77680, 446237): (  # 董岩鋒 — gallery caption: 董岩峰
        "https://q4.itc.cn/q_85,c_zoom,w_600/images03/20260228/ab7df141c2a1451e8e9c192d57246f98.jpeg",
        "446237-chengdu-20260228.jpg",
    ),
    (77680, 446339): (  # 王子铭 — gallery caption: 王子鸣
        "https://q8.itc.cn/q_85,c_zoom,w_600/images03/20260228/b194b7a9c8f042929fe0586bde39b218.jpeg",
        "446339-chengdu-20260228.jpg",
    ),
    (77680, 485367): (  # 李扬 — gallery caption: 李扬
        "https://q7.itc.cn/q_85,c_zoom,w_600/images03/20260228/a28f16b9464a4575a299ab34e149edd5.jpeg",
        "485367-chengdu-20260228.jpg",
    ),
    (77680, 520149): (  # 王东升 — gallery caption: 王东升
        "https://q2.itc.cn/q_85,c_zoom,w_600/images03/20260228/c6180d34d2be4c95b331b191df8f6df3.jpeg",
        "520149-chengdu-20260228.jpg",
    ),
    (77680, 580152): (  # 胡荷韬 — gallery caption: 胡荷韬
        "https://q7.itc.cn/q_85,c_zoom,w_600/images03/20260228/c0f04a023edc45b1a11e0eb8a4fad438.jpeg",
        "580152-chengdu-20260228.jpg",
    ),
    (77680, 580154): (  # 冉伟枫 — gallery caption: 冉伟枫
        "https://q6.itc.cn/q_85,c_zoom,w_600/images03/20260228/52526b46c0434db78a33dcadd0599e76.jpeg",
        "580154-chengdu-20260228.jpg",
    ),
    (77680, 580161): (  # 贺一然 — gallery caption: 贺一然
        "https://q5.itc.cn/q_85,c_zoom,w_600/images03/20260228/745ec0929e2e47bdb34bd02d91847791.jpeg",
        "580161-chengdu-20260228.jpg",
    ),
    (77680, 617755): (  # 唐创 — gallery caption: 唐创
        "https://q3.itc.cn/q_85,c_zoom,w_600/images03/20260228/20fa45a5884b4ebbaf5ebaf48389f2b3.jpeg",
        "617755-chengdu-20260228.jpg",
    ),
    (77680, 617828): (  # 艾孜买提 — gallery caption: 艾孜买提
        "https://q8.itc.cn/q_85,c_zoom,w_600/images03/20260228/0d585a61dce64d819daaefde7c2d1e14.jpeg",
        "617828-chengdu-20260228.jpg",
    ),
    (77680, 617829): (  # 廖荣祥 — gallery caption: 廖荣祥
        "https://q6.itc.cn/q_85,c_zoom,w_600/images03/20260228/96002f1ea9ff4b6994ae0ec1ab1375a9.jpeg",
        "617829-chengdu-20260228.jpg",
    ),
    (77680, 6642603): (  # 茹萨 — gallery caption: 茹萨
        "https://q1.itc.cn/q_85,c_zoom,w_600/images03/20260228/628e02abbd4b45f7aeb635fdc9a35594.jpeg",
        "6642603-chengdu-20260228.jpg",
    ),
    (77680, 6936587): (  # 拜合拉木 — gallery caption: 拜合拉木
        "https://q6.itc.cn/q_85,c_zoom,w_600/images03/20260228/2eb64513c81645108cfcefc1daf1959f.jpeg",
        "6936587-chengdu-20260228.jpg",
    ),
    (77680, 7719460): (  # 李漠雨 — gallery caption: 李漠雨
        "https://q9.itc.cn/q_85,c_zoom,w_600/images03/20260228/4560c0d52c804bcbb948c8944b9fa517.jpeg",
        "7719460-chengdu-20260228.jpg",
    ),
    (77680, 7727346): (  # 席尔瓦 — gallery caption: 韦林顿-席尔瓦
        "https://q1.itc.cn/q_85,c_zoom,w_600/images03/20260228/b758ecd6a4794738bcc89a4e587ce8dd.jpeg",
        "7727346-chengdu-20260228.jpg",
    ),
    (77680, 7732829): (  # 王子腾 — gallery caption: 王子腾
        "https://q5.itc.cn/q_85,c_zoom,w_600/images03/20260228/a9a869df528c4246bfa444434b53a3f2.jpeg",
        "7732829-chengdu-20260228.jpg",
    ),
    (77680, 7941904): (  # 亚历斯祖 — gallery caption: 亚历斯祖
        "https://q9.itc.cn/q_85,c_zoom,w_600/images03/20260228/4d864b626989407681dae6597ea0015e.jpeg",
        "7941904-chengdu-20260228.jpg",
    ),
    (77680, 8218662): (  # 帅惟浩 — gallery caption: 帅惟浩
        "https://q5.itc.cn/q_85,c_zoom,w_600/images03/20260228/c70ec962891d4454b1c64ca54624a75d.jpeg",
        "8218662-chengdu-20260228.jpg",
    ),
    (77680, 8224558): (  # 陈斯亮 — gallery caption: 陈斯亮
        "https://q2.itc.cn/q_85,c_zoom,w_600/images03/20260228/ff8c2515e7e14ffbac12d17030320210.jpeg",
        "8224558-chengdu-20260228.jpg",
    ),
}
