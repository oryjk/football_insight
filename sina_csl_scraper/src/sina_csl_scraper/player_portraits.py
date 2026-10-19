"""Reviewed CSL player portraits, not a live caption/order matching rule.

Source: https://www.sohu.com/picture/991014558 (2026-02-28).
Galleries also include coaches and unnamed staff; only named player photos are used.
Other teams and retained players: docs/csl-player-portraits-2026.md.
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

    # Team 136: https://www.sohu.com/picture/990653196
    (136, 81795): (  # 王刚 — 王刚
        'https://q9.itc.cn/q_85,c_zoom,w_600/images03/20260227/27379453525241d99e993fbd81a51b2f.jpeg',
        '81795-team136-20260227.jpg',
    ),
    (136, 86548): (  # 侯森 — 侯森
        'https://q7.itc.cn/q_85,c_zoom,w_600/images03/20260227/67a5498ecf354832bc7f2803ce4ecc01.jpeg',
        '86548-team136-20260227.jpg',
    ),
    (136, 113053): (  # 张稀哲 — 张稀哲
        'https://q7.itc.cn/q_85,c_zoom,w_600/images03/20260227/91625fa28c5744a1bb00919f7940183d.jpeg',
        '113053-team136-20260227.jpg',
    ),
    (136, 113474): (  # 李磊 — 李磊
        'https://q8.itc.cn/q_85,c_zoom,w_600/images03/20260227/9479f9ba6cc14d3e86c9a4d31dfb1c83.jpeg',
        '113474-team136-20260227.jpg',
    ),
    (136, 180819): (  # 塞尔吉尼奥 — 塞尔吉尼奥
        'https://q7.itc.cn/q_85,c_zoom,w_600/images03/20260227/5871823f04b147fa9667de120a2c9fde.jpeg',
        '180819-team136-20260227.jpg',
    ),
    (136, 205896): (  # 池忠国 — 池忠国
        'https://q0.itc.cn/q_85,c_zoom,w_600/images03/20260227/b4672762f9f8461c8b5ef17c90eb6a5a.jpeg',
        '205896-team136-20260227.jpg',
    ),
    (136, 206326): (  # 曹永竞 — 曹永竞
        'https://q6.itc.cn/q_85,c_zoom,w_600/images03/20260227/7360bffd2f374ae4ae7b2cdf5609d54c.jpeg',
        '206326-team136-20260227.jpg',
    ),
    (136, 223766): (  # 林良铭 — 林良铭
        'https://q1.itc.cn/q_85,c_zoom,w_600/images03/20260227/3881253ff9d4493e9c6b8407a8be08e9.jpeg',
        '223766-team136-20260227.jpg',
    ),
    (136, 246414): (  # 杨立瑜 — 杨立瑜
        'https://q6.itc.cn/q_85,c_zoom,w_600/images03/20260227/60e9ce6784ee4106aea1cc765df501d7.jpeg',
        '246414-team136-20260227.jpg',
    ),
    (136, 429698): (  # 柏杨 — 柏杨
        'https://q0.itc.cn/q_85,c_zoom,w_600/images03/20260227/455d8560f76b4da3b7c18c8a86d49687.jpeg',
        '429698-team136-20260227.jpg',
    ),
    (136, 443258): (  # 阿不都海米提 — 阿不都海米提
        'https://q1.itc.cn/q_85,c_zoom,w_600/images03/20260227/46dc73436f5c42c49162e9e275c6585d.jpeg',
        '443258-team136-20260227.jpg',
    ),
    (136, 443789): (  # 茹子楠 — 茹子楠
        'https://q9.itc.cn/q_85,c_zoom,w_600/images03/20260227/423520371971461e8232a016040b9d11.jpeg',
        '443789-team136-20260227.jpg',
    ),
    (136, 545036): (  # 王禹 — 王禹
        'https://q8.itc.cn/q_85,c_zoom,w_600/images03/20260227/5c3c38a88dd944ceb28ca5035801000d.jpeg',
        '545036-team136-20260227.jpg',
    ),
    (136, 564609): (  # 贾非凡 — 贾非凡
        'https://q3.itc.cn/q_85,c_zoom,w_600/images03/20260227/ee9f3e2584db437ab4175ddb42994b29.jpeg',
        '564609-team136-20260227.jpg',
    ),
    (136, 617722): (  # 努尔艾力 — 努尔艾力-阿巴斯
        'https://q0.itc.cn/q_85,c_zoom,w_600/images03/20260227/b909f5231d9546aa8ea99c9b50d3b8d4.jpeg',
        '617722-team136-20260227.jpg',
    ),
    (136, 5365160): (  # 法比奥 — 法比奥
        'https://q4.itc.cn/q_85,c_zoom,w_600/images03/20260227/b05d7df6f18540c2bcf9ac679dfae6e5.jpeg',
        '5365160-team136-20260227.jpg',
    ),
    (136, 7389830): (  # 范双杰 — 范双杰
        'https://q2.itc.cn/q_85,c_zoom,w_600/images03/20260227/0cfa71e498b14fa9a0a5abb125a8f8de.jpeg',
        '7389830-team136-20260227.jpg',
    ),
    (136, 7418877): (  # 吴少聪 — 吴少聪
        'https://q9.itc.cn/q_85,c_zoom,w_600/images03/20260227/31e69c923cec4c7cb6962711922f3dcf.jpeg',
        '7418877-team136-20260227.jpg',
    ),
    (136, 7724634): (  # 达万 — 达万
        'https://q1.itc.cn/q_85,c_zoom,w_600/images03/20260227/4849fd19d996457b9d5fe44fded956de.jpeg',
        '7724634-team136-20260227.jpg',
    ),
    (136, 7956541): (  # 卢彤鋆 — 卢彤鋆
        'https://q8.itc.cn/q_85,c_zoom,w_600/images03/20260227/b55752fbd25d4e80b5371504b3d72e58.jpeg',
        '7956541-team136-20260227.jpg',
    ),
    (136, 8001070): (  # 拉莫斯 — 吉列尔梅-拉莫斯
        'https://q1.itc.cn/q_85,c_zoom,w_600/images03/20260227/d14b07d06ff64b818f4ca3b735eda77a.jpeg',
        '8001070-team136-20260227.jpg',
    ),
    (136, 8345634): (  # 邓捷夫 — 邓捷夫
        'https://q2.itc.cn/q_85,c_zoom,w_600/images03/20260227/18e2bfd4173849d8af86643dccff1afa.jpeg',
        '8345634-team136-20260227.jpg',
    ),
    (136, 8402902): (  # 恩科洛洛 — 恩科洛洛
        'https://q4.itc.cn/q_85,c_zoom,w_600/images03/20260227/428d9b22cb0e40b497c48de77918418d.jpeg',
        '8402902-team136-20260227.jpg',
    ),
    (136, 8449420): (  # 孔特 — 孔特
        'https://q3.itc.cn/q_85,c_zoom,w_600/images03/20260227/d00460dca4604ec698f7e6ba3ea01b09.jpeg',
        '8449420-team136-20260227.jpg',
    ),
    # Team 143: https://www.sohu.com/picture/992439853
    (143, 113121): (  # 金洋洋 — 金洋洋
        'https://q5.itc.cn/q_85,c_zoom,w_600/images03/20260304/326fc725c6e04aaf8a554e7b07409321.jpeg',
        '113121-team143-20260304.jpg',
    ),
    (143, 113170): (  # 刘君 — 刘君
        'https://q3.itc.cn/q_85,c_zoom,w_600/images03/20260304/1bc5cb59072040d39d54d8c70f4b56c3.jpeg',
        '113170-team143-20260304.jpg',
    ),
    (143, 113203): (  # 宋文杰 — 宋文杰
        'https://q4.itc.cn/q_85,c_zoom,w_600/images03/20260304/51fe666e69624774acc765ad53790ae0.jpeg',
        '113203-team143-20260304.jpg',
    ),
    (143, 113209): (  # 宋龙 — 宋龙
        'https://q8.itc.cn/q_85,c_zoom,w_600/images03/20260304/079c00e802154d97a539dcedfe5c2b9b.jpeg',
        '113209-team143-20260304.jpg',
    ),
    (143, 119080): (  # 孙正傲 — 孙正傲
        'https://q2.itc.cn/q_85,c_zoom,w_600/images03/20260304/39d0969c37194596ac143bdb83daf7a9.jpeg',
        '119080-team143-20260304.jpg',
    ),
    (143, 137944): (  # 刘佳燊 — 刘佳燊
        'https://q5.itc.cn/q_85,c_zoom,w_600/images03/20260304/9a5c8e44a7d4495db375fa9e59b35541.jpeg',
        '137944-team143-20260304.jpg',
    ),
    (143, 155888): (  # 罗森文 — 罗森文
        'https://q8.itc.cn/q_85,c_zoom,w_600/images03/20260304/ba84a8e6451a493587eb5599757a9f73.jpeg',
        '155888-team143-20260304.jpg',
    ),
    (143, 155958): (  # 林创益 — 林创益
        'https://q7.itc.cn/q_85,c_zoom,w_600/images03/20260304/9062bc47536941b994bcfe8249c9af2c.jpeg',
        '155958-team143-20260304.jpg',
    ),
    (143, 230382): (  # 李海龙 — 李海龙
        'https://q0.itc.cn/q_85,c_zoom,w_600/images03/20260304/c9a6660e611641f08ce5fd8314dc1d32.jpeg',
        '230382-team143-20260304.jpg',
    ),
    (143, 249114): (  # 刘军帅 — 刘军帅
        'https://q7.itc.cn/q_85,c_zoom,w_600/images03/20260304/ee28358c0215486ab99e00ad4abc5f08.jpeg',
        '249114-team143-20260304.jpg',
    ),
    (143, 509091): (  # 刘鑫瑜 — 刘鑫瑜
        'https://q6.itc.cn/q_85,c_zoom,w_600/images03/20260304/a6a785614400451e9f6644c3206de1a9.jpeg',
        '509091-team143-20260304.jpg',
    ),
    (143, 576942): (  # 刘国宝 — 刘国宝
        'https://q8.itc.cn/q_85,c_zoom,w_600/images03/20260304/7e1f191b36f5464aad308879c7c8cd8a.jpeg',
        '576942-team143-20260304.jpg',
    ),
    (143, 5366305): (  # 穆斯塔帕 — 穆斯塔帕
        'https://q7.itc.cn/q_85,c_zoom,w_600/images03/20260304/ddabed42b62e4f319c845a355f312ddd.jpeg',
        '5366305-team143-20260304.jpg',
    ),
    (143, 6910002): (  # 李苏达 — 李苏达
        'https://q2.itc.cn/q_85,c_zoom,w_600/images03/20260304/44834e2891a844ab8ad0ac039ba4fa22.jpeg',
        '6910002-team143-20260304.jpg',
    ),
    (143, 6941548): (  # 王峤 — 王峤
        'https://q2.itc.cn/q_85,c_zoom,w_600/images03/20260304/ac6df10641b7462d99ba84016b8f4213.jpeg',
        '6941548-team143-20260304.jpg',
    ),
    (143, 7313484): (  # 金永浩 — 金永浩
        'https://q1.itc.cn/q_85,c_zoom,w_600/images03/20260304/e2a95b75b23d4cb388d3ca15d37948e6.jpeg',
        '7313484-team143-20260304.jpg',
    ),
    (143, 7377355): (  # 颜卓彬 — 颜卓彬
        'https://q0.itc.cn/q_85,c_zoom,w_600/images03/20260304/5e733643e7f84162811c205790c06858.jpeg',
        '7377355-team143-20260304.jpg',
    ),
    (143, 7384399): (  # 郑传辉 — 郑传辉
        'https://q7.itc.cn/q_85,c_zoom,w_600/images03/20260304/6e4c584b60ed48a38d5fd0d2a14a5458.jpeg',
        '7384399-team143-20260304.jpg',
    ),
    (143, 7397711): (  # 车世伟 — 车世伟
        'https://q5.itc.cn/q_85,c_zoom,w_600/images03/20260304/f88febecc8d64c4a95a5fad9ec336f0e.jpeg',
        '7397711-team143-20260304.jpg',
    ),
    (143, 7455523): (  # 埃朱马 — 埃朱马
        'https://q4.itc.cn/q_85,c_zoom,w_600/images03/20260304/c470c5cb4ba0420591bd899e1ee04786.jpeg',
        '7455523-team143-20260304.jpg',
    ),
    (143, 7740924): (  # 杨聪 — 杨聪
        'https://q4.itc.cn/q_85,c_zoom,w_600/images03/20260304/0d3b94f511db472f96617dda0ee946bc.jpeg',
        '7740924-team143-20260304.jpg',
    ),
    (143, 8360352): (  # 安杰尔科维奇 — 安杰尔科维奇
        'https://q3.itc.cn/q_85,c_zoom,w_600/images03/20260304/8c900c8a35f444c58ec449d5a9eb0ccf.jpeg',
        '8360352-team143-20260304.jpg',
    ),
    (143, 8362635): (  # 梅萨乌迪 — 梅萨乌迪
        'https://q1.itc.cn/q_85,c_zoom,w_600/images03/20260304/6f9c8d096816465bb0894599563d7fd9.jpeg',
        '8362635-team143-20260304.jpg',
    ),
    (143, 8398057): (  # 斯特兰德贝里 — 斯特兰德贝里
        'https://q9.itc.cn/q_85,c_zoom,w_600/images03/20260304/61f1b35647734162809907ae1a638f09.jpeg',
        '8398057-team143-20260304.jpg',
    ),
    (143, 8428802): (  # 邝兆镭 — 邝兆镭
        'https://q3.itc.cn/q_85,c_zoom,w_600/images03/20260304/cedebe68decc45caac06f2f5780b5476.jpeg',
        '8428802-team143-20260304.jpg',
    ),
    (143, 8429081): (  # 韦子弦 — 韦子弦
        'https://q4.itc.cn/q_85,c_zoom,w_600/images03/20260304/18a93059482f49c3855fc83ed6bfe212.jpeg',
        '8429081-team143-20260304.jpg',
    ),
    (143, 8443102): (  # 买乌兰 — 买乌兰
        'https://q7.itc.cn/q_85,c_zoom,w_600/images03/20260304/228e66700ffb482ba87c792442a9bdbf.jpeg',
        '8443102-team143-20260304.jpg',
    ),
    (143, 8450365): (  # 叶博亚 — 叶博亚
        'https://q7.itc.cn/q_85,c_zoom,w_600/images03/20260304/8c77b9b6b91e41cf8e0531778d327009.jpeg',
        '8450365-team143-20260304.jpg',
    ),
    (143, 8451611): (  # 江宇涵 — 江宇涵
        'https://q9.itc.cn/q_85,c_zoom,w_600/images03/20260304/0b690fed37c1499ea4279e3279af6683.jpeg',
        '8451611-team143-20260304.jpg',
    ),
    # Team 148: https://www.sohu.com/picture/992415648
    (148, 155935): (  # 郭皓 — 郭皓
        'https://q8.itc.cn/q_85,c_zoom,w_600/images03/20260304/8dc65e2c6a874966ab5b5dd3f5b2bc00.jpeg',
        '155935-team148-20260304.jpg',
    ),
    (148, 155951): (  # 王秋明 — 王秋明
        'https://q6.itc.cn/q_85,c_zoom,w_600/images03/20260304/f55b83bcacae4729b4be216f6295e3c2.jpeg',
        '155951-team148-20260304.jpg',
    ),
    (148, 180436): (  # 巴顿 — 巴顿
        'https://q9.itc.cn/q_85,c_zoom,w_600/images03/20260304/e93f91c994594d8eab9ea0998ba819ee.jpeg',
        '180436-team148-20260304.jpg',
    ),
    (148, 429593): (  # 谢维军 — 谢维军
        'https://q5.itc.cn/q_85,c_zoom,w_600/images03/20260304/98699cce3dcb459982ab76adf7e583aa.jpeg',
        '429593-team148-20260304.jpg',
    ),
    (148, 446338): (  # 杨帆 — 杨帆
        'https://q4.itc.cn/q_85,c_zoom,w_600/images03/20260304/9ed84a36f2cd4fa98fafa2f076920b66.jpeg',
        '446338-team148-20260304.jpg',
    ),
    (148, 446710): (  # 王献钧 — 王献钧
        'https://q4.itc.cn/q_85,c_zoom,w_600/images03/20260304/eb59f304c2224ec1bf6da30c87924128.jpeg',
        '446710-team148-20260304.jpg',
    ),
    (148, 468205): (  # 闫炳良 — 闫炳良
        'https://q4.itc.cn/q_85,c_zoom,w_600/images03/20260304/09953181f9a146f5b83dde2d09719ce8.jpeg',
        '468205-team148-20260304.jpg',
    ),
    (148, 478813): (  # 黄嘉辉 — 黄嘉辉
        'https://q5.itc.cn/q_85,c_zoom,w_600/images03/20260304/3f2917d961c049aca23a15bd92659677.jpeg',
        '478813-team148-20260304.jpg',
    ),
    (148, 478821): (  # 王政豪 — 王政豪
        'https://q8.itc.cn/q_85,c_zoom,w_600/images03/20260304/bafe3da029104128a696e23ec2cd0a06.jpeg',
        '478821-team148-20260304.jpg',
    ),
    (148, 544618): (  # 石炎 — 石炎
        'https://q5.itc.cn/q_85,c_zoom,w_600/images03/20260304/2ac8808a74814884921d8c901be430ac.jpeg',
        '544618-team148-20260304.jpg',
    ),
    (148, 576939): (  # 季胜攀 — 季胜攀
        'https://q5.itc.cn/q_85,c_zoom,w_600/images03/20260304/912d2ba5eaa44a49bf454307640fb919.jpeg',
        '576939-team148-20260304.jpg',
    ),
    (148, 580190): (  # 李永佳 — 李永佳
        'https://q3.itc.cn/q_85,c_zoom,w_600/images03/20260304/64e83923927c4a00bd67cc0d307a00a9.jpeg',
        '580190-team148-20260304.jpg',
    ),
    (148, 6710759): (  # 吴兴涵 — 吴兴涵
        'https://q7.itc.cn/q_85,c_zoom,w_600/images03/20260304/5678db5ef2dd4558bbb0519fae9531b4.jpeg',
        '6710759-team148-20260304.jpg',
    ),
    (148, 6801638): (  # 刘俊贤 — 刘俊贤
        'https://q5.itc.cn/q_85,c_zoom,w_600/images03/20260304/4998bfee993542888b197956ce9a142b.jpeg',
        '6801638-team148-20260304.jpg',
    ),
    (148, 6912435): (  # 孙铭谦 — 孙铭谦
        'https://q9.itc.cn/q_85,c_zoom,w_600/images03/20260304/4efbb17181b043e09a905bb8163e36e4.jpeg',
        '6912435-team148-20260304.jpg',
    ),
    (148, 7249509): (  # 哈达斯 — 哈达斯
        'https://q7.itc.cn/q_85,c_zoom,w_600/images03/20260304/9f043353b6aa477080ad2dcb809415d3.jpeg',
        '7249509-team148-20260304.jpg',
    ),
    (148, 7396148): (  # 齐雨熙 — 齐雨熙
        'https://q1.itc.cn/q_85,c_zoom,w_600/images03/20260304/d04ea05336904296b282ff298154b7cb.jpeg',
        '7396148-team148-20260304.jpg',
    ),
    (148, 7397670): (  # 陈哲宣 — 陈哲宣
        'https://q7.itc.cn/q_85,c_zoom,w_600/images03/20260304/511030bf237d4ab4a5e02e5ceae365ff.jpeg',
        '7397670-team148-20260304.jpg',
    ),
    (148, 7721691): (  # 基莱斯 — 基莱斯
        'https://q2.itc.cn/q_85,c_zoom,w_600/images03/20260304/e8b221d8c56a480fa037811131aa7eee.jpeg',
        '7721691-team148-20260304.jpg',
    ),
    (148, 7722590): (  # 萨尔瓦多 — 萨尔瓦多
        'https://q8.itc.cn/q_85,c_zoom,w_600/images03/20260304/55b1dce28c464092a52fbc20da79b704.jpeg',
        '7722590-team148-20260304.jpg',
    ),
    (148, 7736163): (  # 张皓然 — 张皓然
        'https://q5.itc.cn/q_85,c_zoom,w_600/images03/20260304/63802a2636a543de91adbf7201d3c072.jpeg',
        '7736163-team148-20260304.jpg',
    ),
    (148, 8365944): (  # 科尔多瓦 — 科尔多瓦
        'https://q2.itc.cn/q_85,c_zoom,w_600/images03/20260304/5ba46b0fdb69402c9565e68326532194.jpeg',
        '8365944-team148-20260304.jpg',
    ),
    (148, 8368267): (  # 谢蒂内 — 谢蒂内
        'https://q6.itc.cn/q_85,c_zoom,w_600/images03/20260304/aa49b72e51cc4e2a9f036d63fa0b4aed.jpeg',
        '8368267-team148-20260304.jpg',
    ),
    (148, 8442277): (  # 乃博宁林 — 乃博宁林
        'https://q6.itc.cn/q_85,c_zoom,w_600/images03/20260304/de49dc2e19324bbbacbeceb380f6413d.jpeg',
        '8442277-team148-20260304.jpg',
    ),
    (148, 8442772): (  # 蔡承峻 — 蔡承峻
        'https://q1.itc.cn/q_85,c_zoom,w_600/images03/20260304/81ee3f5557ae4e72810bcd40a5620934.jpeg',
        '8442772-team148-20260304.jpg',
    ),
    (148, 8444526): (  # 刘帅 — 刘帅
        'https://q4.itc.cn/q_85,c_zoom,w_600/images03/20260304/02cb74d003654339bbf0b444492ce0b4.jpeg',
        '8444526-team148-20260304.jpg',
    ),
    (148, 8450071): (  # 李帅琪 — 李帅琪
        'https://q7.itc.cn/q_85,c_zoom,w_600/images03/20260304/f3bb88a1dbed4a2cb47bf4e1e7e2a404.jpeg',
        '8450071-team148-20260304.jpg',
    ),
    # Team 178: https://www.sohu.com/picture/990315580
    (178, 155950): (  # 郑达伦 — 郑达伦
        'https://q9.itc.cn/q_85,c_zoom,w_600/images03/20260226/9705d8a15fb64d46a281353c9f9d9f4b.jpeg',
        '155950-team178-20260226.jpg',
    ),
    (178, 160686): (  # 王上源 — 王上源
        'https://q1.itc.cn/q_85,c_zoom,w_600/images03/20260226/1bb62d2eefe54f30ad09cfebbe2206fa.jpeg',
        '160686-team178-20260226.jpg',
    ),
    (178, 180441): (  # 何超 — 何超
        'https://q2.itc.cn/q_85,c_zoom,w_600/images03/20260226/cf5e1da46b684ed5aa9a3aa7fd23d216.jpeg',
        '180441-team178-20260226.jpg',
    ),
    (178, 180459): (  # 杨阔 — 杨阔
        'https://q8.itc.cn/q_85,c_zoom,w_600/images03/20260226/c7222b546f4a431d9cf28ea9bfab9d82.jpeg',
        '180459-team178-20260226.jpg',
    ),
    (178, 180539): (  # 古斯塔沃 — 古斯塔沃
        'https://q3.itc.cn/q_85,c_zoom,w_600/images03/20260226/0ba20537cd3e4d58a3fc2d577a64d94c.jpeg',
        '180539-team178-20260226.jpg',
    ),
    (178, 205812): (  # 王国明 — 王国明
        'https://q6.itc.cn/q_85,c_zoom,w_600/images03/20260226/35d5bd2755864c09b1d3acbcbc7c8f27.jpeg',
        '205812-team178-20260226.jpg',
    ),
    (178, 446692): (  # 钟义浩 — 钟义浩
        'https://q4.itc.cn/q_85,c_zoom,w_600/images03/20260226/fa08be59488a48f6863f0cf91423009b.jpeg',
        '446692-team178-20260226.jpg',
    ),
    (178, 468207): (  # 杨意林 — 杨意林
        'https://q8.itc.cn/q_85,c_zoom,w_600/images03/20260226/564708719fde4d79a1e8e8108bea0616.jpeg',
        '468207-team178-20260226.jpg',
    ),
    (178, 508435): (  # 阿布拉汗 — 阿布拉汗-哈力克
        'https://q2.itc.cn/q_85,c_zoom,w_600/images03/20260226/3328a498d7cb4071aab3455968840272.jpeg',
        '508435-team178-20260226.jpg',
    ),
    (178, 509806): (  # 史成龙 — 史成龙
        'https://q8.itc.cn/q_85,c_zoom,w_600/images03/20260226/a3e82a0089104750b6526cd8a74684cd.jpeg',
        '509806-team178-20260226.jpg',
    ),
    (178, 617721): (  # 乃比江 — 乃比江
        'https://q0.itc.cn/q_85,c_zoom,w_600/images03/20260226/7c5cdf30e91346159ad63266cc7a936b.jpeg',
        '617721-team178-20260226.jpg',
    ),
    (178, 5367714): (  # 王金帅 — 王金帅
        'https://q7.itc.cn/q_85,c_zoom,w_600/images03/20260226/6887eeef0b4c4713b7a35c620c5a6701.jpeg',
        '5367714-team178-20260226.jpg',
    ),
    (178, 6692270): (  # 迈达纳 — 迈达纳
        'https://q5.itc.cn/q_85,c_zoom,w_600/images03/20260226/8241bd89c16348e09356082b157a59cc.jpeg',
        '6692270-team178-20260226.jpg',
    ),
    (178, 7727639): (  # 马亚 — 卢卡斯-马亚
        'https://q8.itc.cn/q_85,c_zoom,w_600/images03/20260226/79f5de8e9a234e60aeed31be426d15f6.jpeg',
        '7727639-team178-20260226.jpg',
    ),
    # Team 264: https://www.sohu.com/picture/993005456
    (264, 80496): (  # 钱杰给 — 钱杰给
        'https://q5.itc.cn/q_85,c_zoom,w_600/images03/20260305/627086eae977488daebc0d6a44599b47.jpeg',
        '80496-team264-20260305.jpg',
    ),
    (264, 102833): (  # 张佳祺 — 张佳祺
        'https://q4.itc.cn/q_85,c_zoom,w_600/images03/20260305/999b4358cda444fcb12457f213d9e46d.jpeg',
        '102833-team264-20260305.jpg',
    ),
    (264, 113143): (  # 孙国文 — 孙国文
        'https://q7.itc.cn/q_85,c_zoom,w_600/images03/20260305/8f49a4fcfd36405d93b0a121a01cb28a.jpeg',
        '113143-team264-20260305.jpg',
    ),
    (264, 113208): (  # 董春雨 — 董春雨
        'https://q3.itc.cn/q_85,c_zoom,w_600/images03/20260305/b19171597af744d7876c36425dc7fbc0.jpeg',
        '113208-team264-20260305.jpg',
    ),
    (264, 113386): (  # 高迪 — 高迪
        'https://q7.itc.cn/q_85,c_zoom,w_600/images03/20260305/6dd03656c1a54256812fdd0f84a36e61.jpeg',
        '113386-team264-20260305.jpg',
    ),
    (264, 174460): (  # 赵博 — 赵博
        'https://q9.itc.cn/q_85,c_zoom,w_600/images03/20260305/ad44167906084cf9b297c40d57d3c99e.jpeg',
        '174460-team264-20260305.jpg',
    ),
    (264, 180450): (  # 程进 — 程进
        'https://q2.itc.cn/q_85,c_zoom,w_600/images03/20260305/c65a844cc43c466abdfddbdee5962817.jpeg',
        '180450-team264-20260305.jpg',
    ),
    (264, 223981): (  # 吴伟 — 吴伟
        'https://q0.itc.cn/q_85,c_zoom,w_600/images03/20260305/7a6890373c8c46be85a437caeb8d0f54.jpeg',
        '223981-team264-20260305.jpg',
    ),
    (264, 223982): (  # 童磊 — 童磊
        'https://q2.itc.cn/q_85,c_zoom,w_600/images03/20260305/15b029fad79f4668b5f1309dd83b1ae7.jpeg',
        '223982-team264-20260305.jpg',
    ),
    (264, 223983): (  # 王瑒 — 王瑒
        'https://q0.itc.cn/q_85,c_zoom,w_600/images03/20260305/5fa6c87b03304d82b015e8db5925a8cb.jpeg',
        '223983-team264-20260305.jpg',
    ),
    (264, 431094): (  # 卢卡斯 — 卢卡斯
        'https://q9.itc.cn/q_85,c_zoom,w_600/images03/20260305/dff1aeed88814da7824b100e79b27c9f.jpeg',
        '431094-team264-20260305.jpg',
    ),
    (264, 468202): (  # 陶强龙 — 陶强龙
        'https://q4.itc.cn/q_85,c_zoom,w_600/images03/20260305/ff3eb99848264f36b0da6b587d97d886.jpeg',
        '468202-team264-20260305.jpg',
    ),
    (264, 536171): (  # 方昊 — 方昊
        'https://q9.itc.cn/q_85,c_zoom,w_600/images03/20260305/80bd86f1e37f47c58f3b83e24d40d1c3.jpeg',
        '536171-team264-20260305.jpg',
    ),
    (264, 580193): (  # 霍深坪 — 霍深坪
        'https://q9.itc.cn/q_85,c_zoom,w_600/images03/20260305/9218c81ce16b46718137c2f59a0e2b91.jpeg',
        '580193-team264-20260305.jpg',
    ),
    (264, 580255): (  # 刘浩帆 — 刘浩帆
        'https://q1.itc.cn/q_85,c_zoom,w_600/images03/20260305/f27f01298d114bbba2a4babf513ed311.jpeg',
        '580255-team264-20260305.jpg',
    ),
    (264, 608697): (  # 宁方泽 — 宁方泽
        'https://q0.itc.cn/q_85,c_zoom,w_600/images03/20260305/0ff3fb3489cf40d692a10dea79aeca37.jpeg',
        '608697-team264-20260305.jpg',
    ),
    (264, 608699): (  # 徐俊驰 — 徐俊驰
        'https://q1.itc.cn/q_85,c_zoom,w_600/images03/20260305/f0c5fdd3cbe04342b8b8e334999a6933.jpeg',
        '608699-team264-20260305.jpg',
    ),
    (264, 617823): (  # 麻皓齐 — 麻皓齐
        'https://q0.itc.cn/q_85,c_zoom,w_600/images03/20260305/9767b9fbf3b7475db39789603f9c4f24.jpeg',
        '617823-team264-20260305.jpg',
    ),
    (264, 5369140): (  # 鲍盛鑫 — 鲍盛鑫
        'https://q0.itc.cn/q_85,c_zoom,w_600/images03/20260305/3c47355776f14d84ae98e39430ef177a.jpeg',
        '5369140-team264-20260305.jpg',
    ),
    (264, 5393802): (  # 朴镇燮 — 朴镇燮
        'https://q2.itc.cn/q_85,c_zoom,w_600/images03/20260305/ce6c3bab7bb84a519e9b69a6f80ddfd3.jpeg',
        '5393802-team264-20260305.jpg',
    ),
    (264, 7398922): (  # 汪士钦 — 2026年3月4日，浙江杭州，2026中超联赛官方标准照，汪士钦浙江俱乐部绿城大头照。
        'https://q0.itc.cn/q_85,c_zoom,w_600/images03/20260305/f7ce682a7eea40c099cc3f20b5e225ab.jpeg',
        '7398922-team264-20260305.jpg',
    ),
    (264, 7435273): (  # 米特里策 — 米特里策
        'https://q1.itc.cn/q_85,c_zoom,w_600/images03/20260305/c15803ac7cff4ab9bbdbd44eb4a7f76a.jpeg',
        '7435273-team264-20260305.jpg',
    ),
    (264, 7458440): (  # 托利奇 — 托利奇
        'https://q9.itc.cn/q_85,c_zoom,w_600/images03/20260305/16e31cdfd02e402f94bf0ba05edfb96b.jpeg',
        '7458440-team264-20260305.jpg',
    ),
    (264, 7897604): (  # 王鹏博 — 王鹏博
        'https://q7.itc.cn/q_85,c_zoom,w_600/images03/20260305/41537befa3a5476280705a864246f5b2.jpeg',
        '7897604-team264-20260305.jpg',
    ),
    (264, 7940804): (  # 张瑷晖 — 张瑷晖
        'https://q0.itc.cn/q_85,c_zoom,w_600/images03/20260305/a6d1bc9a573d4881b58d865961ea4d68.jpeg',
        '7940804-team264-20260305.jpg',
    ),
    (264, 8155940): (  # 瓜里拉帕 — 瓜里拉帕
        'https://q9.itc.cn/q_85,c_zoom,w_600/images03/20260305/553ca0f06e044e529c6dad44ddf4fedb.jpeg',
        '8155940-team264-20260305.jpg',
    ),
    (264, 8461091): (  # 姜宇轩 — 姜宇轩
        'https://q6.itc.cn/q_85,c_zoom,w_600/images03/20260305/21578ee5a62f41298051148f32671840.jpeg',
        '8461091-team264-20260305.jpg',
    ),
    (264, 8463199): (  # 周文涛 — 周文涛
        'https://q7.itc.cn/q_85,c_zoom,w_600/images03/20260305/72db657b5c434693928491e701d1f493.jpeg',
        '8463199-team264-20260305.jpg',
    ),
    # Team 500: https://www.sohu.com/picture/993142503
    (500, 86131): (  # 弋腾 — 2026年3月5日，云南玉溪，2026中超联赛官方标准照，云南玉昆大头照。弋腾
        'https://q9.itc.cn/q_85,c_zoom,w_600/images03/20260306/285c5f490a3448c1b325bcf7af790aa7.jpeg',
        '86131-team500-20260306.jpg',
    ),
    (500, 156028): (  # 赵宇豪 — 2026年3月5日，云南玉溪，2026中超联赛官方标准照，云南玉昆大头照。赵宇豪
        'https://q1.itc.cn/q_85,c_zoom,w_600/images03/20260306/289146f811d64607839ef79b5a9cfd82.jpeg',
        '156028-team500-20260306.jpg',
    ),
    (500, 185324): (  # 徐新 — 2026年3月5日，云南玉溪，2026中超联赛官方标准照，云南玉昆大头照。徐新
        'https://q1.itc.cn/q_85,c_zoom,w_600/images03/20260306/f2aae7547b644429bd2148fb8164d732.jpeg',
        '185324-team500-20260306.jpg',
    ),
    (500, 205300): (  # 叶楚贵 — 2026年3月5日，云南玉溪，2026中超联赛官方标准照，云南玉昆大头照。叶楚贵
        'https://q3.itc.cn/q_85,c_zoom,w_600/images03/20260306/ae03bc8615b04c469d38a329232e7617.jpeg',
        '205300-team500-20260306.jpg',
    ),
    (500, 249382): (  # 黄紫昌 — 2026年3月5日，云南玉溪，2026中超联赛官方标准照，云南玉昆大头照。黄紫昌
        'https://q0.itc.cn/q_85,c_zoom,w_600/images03/20260306/5720fe902a9043a6a2c949ef0718b5b8.jpeg',
        '249382-team500-20260306.jpg',
    ),
    (500, 456290): (  # 奥斯卡 — 2026年3月5日，云南玉溪，2026中超联赛官方标准照，云南玉昆大头照。奥斯卡
        'https://q1.itc.cn/q_85,c_zoom,w_600/images03/20260306/7544953e1bdc41f19f9af071aae1d9cb.jpeg',
        '456290-team500-20260306.jpg',
    ),
    (500, 478858): (  # 鲍亚雄 — 2026年3月5日，云南玉溪，2026中超联赛官方标准照，云南玉昆大头照。鲍亚雄
        'https://q9.itc.cn/q_85,c_zoom,w_600/images03/20260306/ee1ab5ce877a413baf2414a0796224c3.jpeg',
        '478858-team500-20260306.jpg',
    ),
    (500, 489981): (  # 孙学龙 — 2026年3月5日，云南玉溪，2026中超联赛官方标准照，云南玉昆大头照。孙学龙
        'https://q6.itc.cn/q_85,c_zoom,w_600/images03/20260306/4d21696c36a24ba380db82e666062ab4.jpeg',
        '489981-team500-20260306.jpg',
    ),
    (500, 536373): (  # 张祥硕 — 2026年3月5日，云南玉溪，2026中超联赛官方标准照，云南玉昆大头照。张祥硕
        'https://q7.itc.cn/q_85,c_zoom,w_600/images03/20260306/c60599b5f995408cbc1f52f9e786d14e.jpeg',
        '536373-team500-20260306.jpg',
    ),
    (500, 536482): (  # 杨梓豪 — 2026年3月5日，云南玉溪，2026中超联赛官方标准照，云南玉昆大头照。杨梓豪
        'https://q3.itc.cn/q_85,c_zoom,w_600/images03/20260306/58aa7e7050024a79874322b575b1ca20.jpeg',
        '536482-team500-20260306.jpg',
    ),
    (500, 607002): (  # 费尔南多 — 2026年3月5日，云南玉溪，2026中超联赛官方标准照，云南玉昆大头照。费尔南多
        'https://q3.itc.cn/q_85,c_zoom,w_600/images03/20260306/fdfae16cb9394a719c571591595d7612.jpeg',
        '607002-team500-20260306.jpg',
    ),
    (500, 5399759): (  # 徐宏杰 — 2026年3月5日，云南玉溪，2026中超联赛官方标准照，云南玉昆大头照。徐宏杰
        'https://q3.itc.cn/q_85,c_zoom,w_600/images03/20260306/79e1165e1d694f1f8c1732dd7301f687.jpeg',
        '5399759-team500-20260306.jpg',
    ),
    (500, 7150362): (  # 布尔克 — 2026年3月5日，云南玉溪，2026中超联赛官方标准照，云南玉昆大头照。布尔克
        'https://q9.itc.cn/q_85,c_zoom,w_600/images03/20260306/dcfb7e8833484c739712f89f81e5cecf.jpeg',
        '7150362-team500-20260306.jpg',
    ),
    (500, 7397040): (  # 韩子龙 — 2026年3月5日，云南玉溪，2026中超联赛官方标准照，云南玉昆大头照。韩子龙
        'https://q8.itc.cn/q_85,c_zoom,w_600/images03/20260306/51bfdf07ad8741eb8681794ceef4f307.jpeg',
        '7397040-team500-20260306.jpg',
    ),
    (500, 7399810): (  # 约尼查 — 2026年3月5日，云南玉溪，2026中超联赛官方标准照，云南玉昆大头照。约尼查
        'https://q4.itc.cn/q_85,c_zoom,w_600/images03/20260306/1aa70a45aae34bf7a5614fe8a080bef5.jpeg',
        '7399810-team500-20260306.jpg',
    ),
    (500, 7400692): (  # 张宸梁 — 2026年3月5日，云南玉溪，2026中超联赛官方标准照，云南玉昆大头照。张宸梁
        'https://q5.itc.cn/q_85,c_zoom,w_600/images03/20260306/034a9dab418840829293a8605707bb2c.jpeg',
        '7400692-team500-20260306.jpg',
    ),
    (500, 7400725): (  # 余鉴贤 — 2026年3月5日，云南玉溪，2026中超联赛官方标准照，云南玉昆大头照。余鉴贤
        'https://q1.itc.cn/q_85,c_zoom,w_600/images03/20260306/78269d772d694f289c01e7978b7f9355.jpeg',
        '7400725-team500-20260306.jpg',
    ),
    (500, 7718971): (  # 侯永永 — 2026年3月5日，云南玉溪，2026中超联赛官方标准照，云南玉昆大头照。侯永永
        'https://q6.itc.cn/q_85,c_zoom,w_600/images03/20260306/df101840004f4700beee560a2e517225.jpeg',
        '7718971-team500-20260306.jpg',
    ),
    (500, 8313210): (  # 克莱伯 — 2026年3月5日，云南玉溪，2026中超联赛官方标准照，云南玉昆大头照。克莱伯
        'https://q6.itc.cn/q_85,c_zoom,w_600/images03/20260306/a500cddb02ae4fbf8ba22ebe62f1c0f5.jpeg',
        '8313210-team500-20260306.jpg',
    ),
    (500, 8316982): (  # 卡约 — 2026年3月5日，云南玉溪，2026中超联赛官方标准照，云南玉昆大头照。卡约
        'https://q7.itc.cn/q_85,c_zoom,w_600/images03/20260306/806911e4a6704560a47250f222afa22f.jpeg',
        '8316982-team500-20260306.jpg',
    ),
    (500, 8418154): (  # 布尼亚明 — 2026年3月5日，云南玉溪，2026中超联赛官方标准照，云南玉昆大头照。布尼亚明
        'https://q6.itc.cn/q_85,c_zoom,w_600/images03/20260306/720898e17a504543802a3171ed6dc94b.jpeg',
        '8418154-team500-20260306.jpg',
    ),
    (500, 8445039): (  # 王星皓 — 2026年3月5日，云南玉溪，2026中超联赛官方标准照，云南玉昆大头照。王星皓
        'https://q4.itc.cn/q_85,c_zoom,w_600/images03/20260306/2a53a2c84db24342b3ec25bb735042f4.jpeg',
        '8445039-team500-20260306.jpg',
    ),
    (500, 8446589): (  # 荣家辉 — 2026年3月5日，云南玉溪，2026中超联赛官方标准照，云南玉昆大头照。荣家辉
        'https://q8.itc.cn/q_85,c_zoom,w_600/images03/20260306/33920bf660304cb19eb2ce414f954bb0.jpeg',
        '8446589-team500-20260306.jpg',
    ),
    # Team 501: https://www.sohu.com/a/991429735_585510
    (501, 83289): (  # 斯坦丘 — 10-斯坦丘，尼古拉·斯坦丘（Nicolae Claudiu Stanciu），国籍：罗马尼亚。
        'https://q8.itc.cn/q_85,c_zoom,w_600/images01/20260302/df5de99a732a4f129b9f85ba1f275b39.jpeg',
        '83289-team501-20260302.jpg',
    ),
    (501, 113139): (  # 吕鹏 — 队长放在最前面，21-吕鹏（晚池注：号码仅供参考，最终以实际为准。）
        'https://q1.itc.cn/q_85,c_zoom,w_600/images01/20260302/8f130dab24e14e61b3a5a8b64b418375.jpeg',
        '113139-team501-20260302.jpg',
    ),
    (501, 113336): (  # 毕津浩 — 毕津浩
        'https://q1.itc.cn/q_85,c_zoom,w_600/images01/20260302/74f48c40963241d3a198674e3c52ad2b.jpeg',
        '113336-team501-20260302.jpg',
    ),
    (501, 113406): (  # 阿利米 — 4-阿利米，阿利米（Isnik Alimi），国籍：北马其顿。
        'https://q6.itc.cn/q_85,c_zoom,w_600/images01/20260302/31794d52a09649708f34c3535ee0429c.jpeg',
        '113406-team501-20260302.jpg',
    ),
    (501, 114677): (  # 马莱莱 — 11-马莱莱，赛法斯·马莱莱（Cephas Malele）。国籍：瑞士，安哥拉。
        'https://q4.itc.cn/q_85,c_zoom,w_600/images01/20260302/cd6ba586a9914018aedd5d7ba0b7f8e4.jpeg',
        '114677-team501-20260302.jpg',
    ),
    (501, 153460): (  # 阿奇姆彭 — 30-阿奇姆彭，弗兰克·阿奇姆彭（Frank Acheampong），国籍：加纳。
        'https://q2.itc.cn/q_85,c_zoom,w_600/images01/20260302/ae327787ff044baf8c267dda828895f6.jpeg',
        '153460-team501-20260302.jpg',
    ),
    (501, 167854): (  # 罗竞 — 7-罗竞
        'https://q3.itc.cn/q_85,c_zoom,w_600/images01/20260302/49fb4f1aef5744b784e878789df27f7b.jpeg',
        '167854-team501-20260302.jpg',
    ),
    (501, 205801): (  # 冯劲 — 44-冯劲
        'https://q9.itc.cn/q_85,c_zoom,w_600/images01/20260302/5ff9f33eb1304de7b468adfee51a9de3.jpeg',
        '205801-team501-20260302.jpg',
    ),
    (501, 247195): (  # 张华晨 — 8-张华晨
        'https://q4.itc.cn/q_85,c_zoom,w_600/images01/20260302/ae5885b2008f4ce0a26f757e3215ce52.jpeg',
        '247195-team501-20260302.jpg',
    ),
    (501, 247906): (  # 温家宝 — 温姓球员家宝
        'https://q0.itc.cn/q_85,c_zoom,w_600/images01/20260302/46ad65d4577c40bc952036a04e044f00.jpeg',
        '247906-team501-20260302.jpg',
    ),
    (501, 536344): (  # 廖锦涛 — 40-廖锦涛
        'https://q8.itc.cn/q_85,c_zoom,w_600/images01/20260302/f729b31d6d3146248e472da5cf2cf664.jpeg',
        '536344-team501-20260302.jpg',
    ),
    (501, 581253): (  # 吕焯毅 — 38-吕焯毅
        'https://q6.itc.cn/q_85,c_zoom,w_600/images01/20260302/405f7f61c7f743ebaf768181bccdd15b.jpeg',
        '581253-team501-20260302.jpg',
    ),
    (501, 6687909): (  # 马马杜 — 2-马马杜，马马杜·塞库·特拉奥雷（Mamadou Sekou Traoré），国籍：马里。
        'https://q1.itc.cn/q_85,c_zoom,w_600/images01/20260302/16583b3b3f154302a5ada3f064711077.jpeg',
        '6687909-team501-20260302.jpg',
    ),
    (501, 6941307): (  # 宋岳 — 6-宋岳
        'https://q0.itc.cn/q_85,c_zoom,w_600/images01/20260302/ea58bd1eae7a4b019c460ffdff98ba6c.jpeg',
        '6941307-team501-20260302.jpg',
    ),
    (501, 7393516): (  # 黄子豪 — 26-黄子豪
        'https://q0.itc.cn/q_85,c_zoom,w_600/images01/20260302/348abba025ee4bcf85b23bbd8ba08100.jpeg',
        '7393516-team501-20260302.jpg',
    ),
    (501, 7394283): (  # 葛彭 — 1-葛彭
        'https://q2.itc.cn/q_85,c_zoom,w_600/images01/20260302/78d3f71e801347c1b8047d7e4ac79cd1.jpeg',
        '7394283-team501-20260302.jpg',
    ),
    (501, 7397175): (  # 刘伟国 — 20-刘伟国
        'https://q7.itc.cn/q_85,c_zoom,w_600/images01/20260302/cbff2b6fe16d44e3859f6a4748992473.jpeg',
        '7397175-team501-20260302.jpg',
    ),
    (501, 7397236): (  # 杨铭锐 — 27-杨铭锐（U21）
        'https://q7.itc.cn/q_85,c_zoom,w_600/images01/20260302/16757e099772462399dfa1074df2b57e.jpeg',
        '7397236-team501-20260302.jpg',
    ),
    (501, 7397736): (  # 黄山 — 23-黄山（U21）
        'https://q8.itc.cn/q_85,c_zoom,w_600/images01/20260302/96b08169a484416f9663bcf7a0734c0a.jpeg',
        '7397736-team501-20260302.jpg',
    ),
    (501, 7400410): (  # 朱鹏宇 — 16-朱鹏宇（U21）
        'https://q8.itc.cn/q_85,c_zoom,w_600/images01/20260302/d85d2cbb3c6e4b87b67b13977346f9b7.jpeg',
        '7400410-team501-20260302.jpg',
    ),
    (501, 7401295): (  # 毛伟杰 — 22-毛伟杰（U21）
        'https://q8.itc.cn/q_85,c_zoom,w_600/images01/20260302/02065774463844ca9315439904e21ce2.jpeg',
        '7401295-team501-20260302.jpg',
    ),
    (501, 7402086): (  # 曹海清 — 33-曹海清
        'https://q6.itc.cn/q_85,c_zoom,w_600/images01/20260302/6f101fa00c7d49e491d749121435d562.jpeg',
        '7402086-team501-20260302.jpg',
    ),
    (501, 7956862): (  # 彭顺杰 — 24-彭顺杰（U21）
        'https://q0.itc.cn/q_85,c_zoom,w_600/images01/20260302/dc801280583c44388717a12f941c7f60.jpeg',
        '7956862-team501-20260302.jpg',
    ),
    (501, 8379235): (  # 孙康博 — 28-孙康博（U21）
        'https://q9.itc.cn/q_85,c_zoom,w_600/images01/20260302/d5a7789fdeb94aae974eb3be36057e27.jpeg',
        '8379235-team501-20260302.jpg',
    ),
    (501, 8400840): (  # 张城瑞 — 张城瑞（U21）
        'https://q3.itc.cn/q_85,c_zoom,w_600/images01/20260302/33b803abaf884a3e953309474ca9be95.jpeg',
        '8400840-team501-20260302.jpg',
    ),
    # Team 507: https://www.sohu.com/picture/992519974
    (507, 113062): (  # 李提香 — 李提香
        'https://q8.itc.cn/q_85,c_zoom,w_600/images03/20260304/234888915f2641aea4a5ce3e3b1bb60e.jpeg',
        '113062-team507-20260304.jpg',
    ),
    (507, 115113): (  # 臧一锋 — 臧一锋
        'https://q2.itc.cn/q_85,c_zoom,w_600/images03/20260304/8a951aec44744eb39a50fbf869546faf.jpeg',
        '115113-team507-20260304.jpg',
    ),
    (507, 155890): (  # 韩镕泽 — 韩镕泽
        'https://q8.itc.cn/q_85,c_zoom,w_600/images03/20260304/c42017f7e313447ba2c2346685c92d7c.jpeg',
        '155890-team507-20260304.jpg',
    ),
    (507, 180409): (  # 许东 — 许东
        'https://q6.itc.cn/q_85,c_zoom,w_600/images03/20260304/1b536aa0eb92431c8e0b88216cca0be4.jpeg',
        '180409-team507-20260304.jpg',
    ),
    (507, 205009): (  # 张岩 — 张岩
        'https://q7.itc.cn/q_85,c_zoom,w_600/images03/20260304/37181305ab01418da5ba2642e44f9299.jpeg',
        '205009-team507-20260304.jpg',
    ),
    (507, 210564): (  # 潘喜明 — 潘喜明
        'https://q5.itc.cn/q_85,c_zoom,w_600/images03/20260304/9840dd2cffe64c0f9ba273af957a8ba8.jpeg',
        '210564-team507-20260304.jpg',
    ),
    (507, 224063): (  # 田依浓 — 田依浓
        'https://q6.itc.cn/q_85,c_zoom,w_600/images03/20260304/210a6ca789044582a75b2e93ff12f19e.jpeg',
        '224063-team507-20260304.jpg',
    ),
    (507, 249225): (  # 元敏诚 — 元敏诚
        'https://q0.itc.cn/q_85,c_zoom,w_600/images03/20260304/b141ec3788f540fb898fceacc3d2c841.jpeg',
        '249225-team507-20260304.jpg',
    ),
    (507, 423970): (  # 陈彬彬 — 陈彬彬
        'https://q4.itc.cn/q_85,c_zoom,w_600/images03/20260304/ca729a8ab8824f1c869051d9858a7974.jpeg',
        '423970-team507-20260304.jpg',
    ),
    (507, 437523): (  # 严鼎皓 — 严鼎皓
        'https://q0.itc.cn/q_85,c_zoom,w_600/images03/20260304/392d2f5d180348b3b83f4f6bc0d00cc5.jpeg',
        '437523-team507-20260304.jpg',
    ),
    (507, 477925): (  # 迪力穆拉提 — 迪力穆拉提
        'https://q6.itc.cn/q_85,c_zoom,w_600/images03/20260304/61d2aebcbd914df49f23ae616073aa9d.jpeg',
        '477925-team507-20260304.jpg',
    ),
    (507, 502384): (  # 热菲尼奥 — 热菲尼奥
        'https://q5.itc.cn/q_85,c_zoom,w_600/images03/20260304/066ff3b66c934c439cb11aa0182b71a1.jpeg',
        '502384-team507-20260304.jpg',
    ),
    (507, 577810): (  # 田玉达 — 田玉达
        'https://q6.itc.cn/q_85,c_zoom,w_600/images03/20260304/f685df41361a4c968ee6051e6c36fecc.jpeg',
        '577810-team507-20260304.jpg',
    ),
    (507, 6884514): (  # 李浩然 — 李浩然
        'https://q5.itc.cn/q_85,c_zoom,w_600/images03/20260304/b86a5676fbad49cabe8a7aebb1954a7e.jpeg',
        '6884514-team507-20260304.jpg',
    ),
    (507, 7392356): (  # 庞盛翰 — 庞盛翰
        'https://q7.itc.cn/q_85,c_zoom,w_600/images03/20260304/b9be5c65861046aeb043bdd8608f5682.jpeg',
        '7392356-team507-20260304.jpg',
    ),
    (507, 7394098): (  # 高嘉润 — 高嘉润
        'https://q2.itc.cn/q_85,c_zoom,w_600/images03/20260304/94dc4f96bf8c4dac9754beec73a9eedf.jpeg',
        '7394098-team507-20260304.jpg',
    ),
    (507, 7395141): (  # 田德澳 — 田德澳
        'https://q8.itc.cn/q_85,c_zoom,w_600/images03/20260304/b72ac8eb3a42477e8e36d59bcea7f00c.jpeg',
        '7395141-team507-20260304.jpg',
    ),
    (507, 7396563): (  # 安以恩 — 安以恩
        'https://q5.itc.cn/q_85,c_zoom,w_600/images03/20260304/8603c2f146d64118acd56cf5ddb16c76.jpeg',
        '7396563-team507-20260304.jpg',
    ),
    (507, 8434151): (  # 郑家骋 — 郑家骋
        'https://q2.itc.cn/q_85,c_zoom,w_600/images03/20260304/38ad155f1d84468b952b5cc3030fc7e9.jpeg',
        '8434151-team507-20260304.jpg',
    ),
    (507, 8437231): (  # 姆本扎 — 姆本扎
        'https://q6.itc.cn/q_85,c_zoom,w_600/images03/20260304/1c93bf3cd22743758f92e2237e669361.jpeg',
        '8437231-team507-20260304.jpg',
    ),
    (507, 8447930): (  # 费利佩 — 费利佩
        'https://q9.itc.cn/q_85,c_zoom,w_600/images03/20260304/9279b845ba794834a92244a0177b0ac6.jpeg',
        '8447930-team507-20260304.jpg',
    ),
    (507, 8454755): (  # 李铈琦 — 李铈琦
        'https://q3.itc.cn/q_85,c_zoom,w_600/images03/20260304/fab6f05ff8cd43588030ffe61d5adafd.jpeg',
        '8454755-team507-20260304.jpg',
    ),
    # Team 41300: https://www.sohu.com/picture/992953413
    (41300, 81801): (  # 张琳芃 — 张琳芃
        'https://q9.itc.cn/q_85,c_zoom,w_600/images03/20260305/cf33bbb7230a4d09a24d1cf393d17dd2.jpeg',
        '81801-team41300-20260305.jpg',
    ),
    (41300, 155917): (  # 颜骏凌 — 颜骏凌
        'https://q0.itc.cn/q_85,c_zoom,w_600/images03/20260305/388a16163e804ae5816bb1b9dbf3fe61.jpeg',
        '155917-team41300-20260305.jpg',
    ),
    (41300, 155922): (  # 王燊超 — 王燊超
        'https://q5.itc.cn/q_85,c_zoom,w_600/images03/20260305/f45ef432dcfe4b3dad4a9e6d32cc5365.jpeg',
        '155922-team41300-20260305.jpg',
    ),
    (41300, 180438): (  # 岳鑫 — 岳鑫
        'https://q8.itc.cn/q_85,c_zoom,w_600/images03/20260305/246294fb24ce4d7cae3505456f2d3f8c.jpeg',
        '180438-team41300-20260305.jpg',
    ),
    (41300, 180500): (  # 杨世元 — 杨世元
        'https://q4.itc.cn/q_85,c_zoom,w_600/images03/20260305/f0d2eb899f19498abe13a3da447de968.jpeg',
        '180500-team41300-20260305.jpg',
    ),
    (41300, 230384): (  # 魏震 — 魏震
        'https://q7.itc.cn/q_85,c_zoom,w_600/images03/20260305/7c7c0a960fbe496ea76db6db8c14b716.jpeg',
        '230384-team41300-20260305.jpg',
    ),
    (41300, 248600): (  # 陈威 — 陈威
        'https://q9.itc.cn/q_85,c_zoom,w_600/images03/20260305/08829869b3bc4ca393107da2d62a6e2a.jpeg',
        '248600-team41300-20260305.jpg',
    ),
    (41300, 249437): (  # 刘若钒 — 刘若钒
        'https://q2.itc.cn/q_85,c_zoom,w_600/images03/20260305/8e1cf7a46c7c4274a763032e7bd448b9.jpeg',
        '249437-team41300-20260305.jpg',
    ),
    (41300, 431596): (  # 张源 — 张源
        'https://q6.itc.cn/q_85,c_zoom,w_600/images03/20260305/0453bc1a656344a0bad58c5e0c1f7b94.jpeg',
        '431596-team41300-20260305.jpg',
    ),
    (41300, 442956): (  # 李帅 — 李帅
        'https://q4.itc.cn/q_85,c_zoom,w_600/images03/20260305/c8b5be776ecf466f8f9d57772a38f7c0.jpeg',
        '442956-team41300-20260305.jpg',
    ),
    (41300, 453931): (  # 王振澳 — 王振澳
        'https://q8.itc.cn/q_85,c_zoom,w_600/images03/20260305/133b7f25f6214a51b3922507a262829b.jpeg',
        '453931-team41300-20260305.jpg',
    ),
    (41300, 501292): (  # 莱昂纳多 — 莱昂纳多
        'https://q1.itc.cn/q_85,c_zoom,w_600/images03/20260305/30289d04c2f84cf6bf9e059e07eb9870.jpeg',
        '501292-team41300-20260305.jpg',
    ),
    (41300, 502586): (  # 刘祝润 — 刘祝润
        'https://q4.itc.cn/q_85,c_zoom,w_600/images03/20260305/f17ec4ba62f4411e846f4db09c8213e2.jpeg',
        '502586-team41300-20260305.jpg',
    ),
    (41300, 6799982): (  # 卢永涛 — 卢永涛
        'https://q4.itc.cn/q_85,c_zoom,w_600/images03/20260305/e50b2ef213c245318c77bf5000edb47f.jpeg',
        '6799982-team41300-20260305.jpg',
    ),
    (41300, 6816666): (  # 傅欢 — 傅欢
        'https://q3.itc.cn/q_85,c_zoom,w_600/images03/20260305/996e1559eb0640f0b54fc8e92edffdf7.jpeg',
        '6816666-team41300-20260305.jpg',
    ),
    (41300, 6859435): (  # 鲍世蒙 — 鲍世蒙
        'https://q6.itc.cn/q_85,c_zoom,w_600/images03/20260305/33e98712158b43ba9f793a3393c9065c.jpeg',
        '6859435-team41300-20260305.jpg',
    ),
    (41300, 6910373): (  # 吾米提江 — 吾米提江
        'https://q2.itc.cn/q_85,c_zoom,w_600/images03/20260305/9fdf946f1b044a1594da77e58084281e.jpeg',
        '6910373-team41300-20260305.jpg',
    ),
    (41300, 6936062): (  # 安永佳 — 安永佳
        'https://q1.itc.cn/q_85,c_zoom,w_600/images03/20260305/374fb02b3a3e4b4c8fc69d63d5db8a5e.jpeg',
        '6936062-team41300-20260305.jpg',
    ),
    (41300, 7377683): (  # 克劳德 — 让-克劳德
        'https://q6.itc.cn/q_85,c_zoom,w_600/images03/20260305/b5fb90adbc3b4b01abb9459d773fce28.jpeg',
        '7377683-team41300-20260305.jpg',
    ),
    (41300, 7471589): (  # 李新翔 — 李新翔
        'https://q8.itc.cn/q_85,c_zoom,w_600/images03/20260305/4728bd5d0e9c44cab5a4d96a765bc7d0.jpeg',
        '7471589-team41300-20260305.jpg',
    ),
    (41300, 7712350): (  # 维塔尔 — 马特乌斯-维塔尔
        'https://q7.itc.cn/q_85,c_zoom,w_600/images03/20260305/f418d52856c54bbea87ddd55dd2e5fef.jpeg',
        '7712350-team41300-20260305.jpg',
    ),
    (41300, 7714241): (  # 加布里埃尔 — 加布里埃尔
        'https://q1.itc.cn/q_85,c_zoom,w_600/images03/20260305/89a17415bd374b36a5ad6c8a6584902d.jpeg',
        '7714241-team41300-20260305.jpg',
    ),
    (41300, 7714877): (  # 李智良 — 李智良
        'https://q5.itc.cn/q_85,c_zoom,w_600/images03/20260305/3efdca066d83418d8e90d41c66fc22af.jpeg',
        '7714877-team41300-20260305.jpg',
    ),
    (41300, 7721098): (  # 杨希 — 杨希
        'https://q8.itc.cn/q_85,c_zoom,w_600/images03/20260305/584f361391624441bb41b9ebb740d370.jpeg',
        '7721098-team41300-20260305.jpg',
    ),
    (41300, 7736616): (  # 蒯纪闻 — 蒯纪闻
        'https://q2.itc.cn/q_85,c_zoom,w_600/images03/20260305/fd6b64a7bee048328b0f3b5a046052ea.jpeg',
        '7736616-team41300-20260305.jpg',
    ),
    (41300, 8406720): (  # 张宇航 — 张宇航
        'https://q7.itc.cn/q_85,c_zoom,w_600/images03/20260305/30a362db1e97497b861ed32bbe555319.jpeg',
        '8406720-team41300-20260305.jpg',
    ),
    (41300, 8411126): (  # 王东承 — 王东承
        'https://q5.itc.cn/q_85,c_zoom,w_600/images03/20260305/bfe02a14d9ce48b99adaf1db452b9a82.jpeg',
        '8411126-team41300-20260305.jpg',
    ),
    (41300, 8444852): (  # 安佩姆 — 安佩姆
        'https://q7.itc.cn/q_85,c_zoom,w_600/images03/20260305/696c3dd680ad4215bd58c55bb88927ed.jpeg',
        '8444852-team41300-20260305.jpg',
    ),
    # Team 87942: https://www.sohu.com/picture/990261583
    (87942, 113481): (  # 姜至鹏 — 姜至鹏
        'https://q6.itc.cn/q_85,c_zoom,w_600/images03/20260226/8565033cb63349969b51a43e6c1cc951.jpeg',
        '113481-team87942-20260226.jpg',
    ),
    (87942, 167903): (  # 周定洋 — 周定洋
        'https://q6.itc.cn/q_85,c_zoom,w_600/images03/20260226/7a415370c0b746dbae59a6f6b86f9ec0.jpeg',
        '167903-team87942-20260226.jpg',
    ),
    (87942, 429570): (  # 卡尔采夫 — 卡尔采夫
        'https://q6.itc.cn/q_85,c_zoom,w_600/images03/20260226/5fa46a7251824cef9ce593f1f403de21.jpeg',
        '429570-team87942-20260226.jpg',
    ),
    (87942, 430859): (  # 戴伟浚 — 戴伟浚
        'https://q6.itc.cn/q_85,c_zoom,w_600/images03/20260226/0fde54221a204205a440e630612db521.jpeg',
        '430859-team87942-20260226.jpg',
    ),
    (87942, 442946): (  # 姚均晟 — 姚均晟
        'https://q7.itc.cn/q_85,c_zoom,w_600/images03/20260226/9d0b52463e2a43f49c6508d94b42276d.jpeg',
        '442946-team87942-20260226.jpg',
    ),
    (87942, 509095): (  # 杨一鸣 — 杨一鸣
        'https://q1.itc.cn/q_85,c_zoom,w_600/images03/20260226/8d8a9cf47c1f49f0bcf9b387e02e70e7.jpeg',
        '509095-team87942-20260226.jpg',
    ),
    (87942, 536472): (  # 魏黾哲 — 魏黾哲
        'https://q0.itc.cn/q_85,c_zoom,w_600/images03/20260226/70e21194678042aa916660f62dad85a9.jpeg',
        '536472-team87942-20260226.jpg',
    ),
    (87942, 7728487): (  # 申桓铭 — 申桓铭
        'https://q5.itc.cn/q_85,c_zoom,w_600/images03/20260226/3190f8a86bb24bdb9e85e0cb7b1a8b5a.jpeg',
        '7728487-team87942-20260226.jpg',
    ),
    (87942, 7741264): (  # 张皓楠 — 张皓楠
        'https://q3.itc.cn/q_85,c_zoom,w_600/images03/20260226/5e608dbd109d40e78b18184953729a8c.jpeg',
        '7741264-team87942-20260226.jpg',
    ),
    # Team 123380: https://www.sohu.com/picture/992394253
    (123380, 80723): (  # 张呈栋 — 张呈栋
        'https://q4.itc.cn/q_85,c_zoom,w_600/images03/20260304/381b0597f74b41a796c5eaaf03099d3c.jpeg',
        '80723-team123380-20260304.jpg',
    ),
    (123380, 113078): (  # 孙捷 — 孙捷
        'https://q7.itc.cn/q_85,c_zoom,w_600/images03/20260304/e3bdec5c9dc94bfcabb55c23b7a3d517.jpeg',
        '113078-team123380-20260304.jpg',
    ),
    (123380, 113127): (  # 赵宏略 — 赵宏略
        'https://q1.itc.cn/q_85,c_zoom,w_600/images03/20260304/15a90284f07447d3b56b7c024cc0a4f0.jpeg',
        '113127-team123380-20260304.jpg',
    ),
    (123380, 113298): (  # 董宇 — 董宇
        'https://q2.itc.cn/q_85,c_zoom,w_600/images03/20260304/9df4b16ed56b4aff9add9934d143ef23.jpeg',
        '113298-team123380-20260304.jpg',
    ),
    (123380, 205815): (  # 王鹏 — 王鹏
        'https://q2.itc.cn/q_85,c_zoom,w_600/images03/20260304/465c4b9da85947639b4305bab3e72068.jpeg',
        '205815-team123380-20260304.jpg',
    ),
    (123380, 244327): (  # 戴维森 — 戴维森
        'https://q4.itc.cn/q_85,c_zoom,w_600/images03/20260304/975cec7ee6ea42b3a463cc97551414fa.jpeg',
        '244327-team123380-20260304.jpg',
    ),
    (123380, 248598): (  # 刘世博 — 刘世博
        'https://q4.itc.cn/q_85,c_zoom,w_600/images03/20260304/5899361b644047df865fbfac20fea5f5.jpeg',
        '248598-team123380-20260304.jpg',
    ),
    (123380, 484544): (  # 阿齐兹 — 阿齐兹
        'https://q9.itc.cn/q_85,c_zoom,w_600/images03/20260304/208598576bb74b3ea4eb75d4f7857374.jpeg',
        '484544-team123380-20260304.jpg',
    ),
    (123380, 536066): (  # 何龙海 — 何龙海
        'https://q7.itc.cn/q_85,c_zoom,w_600/images03/20260304/f1b002346d50474699ff15d99a7b486a.jpeg',
        '536066-team123380-20260304.jpg',
    ),
    (123380, 6878244): (  # 丁海峰 — 丁海峰
        'https://q6.itc.cn/q_85,c_zoom,w_600/images03/20260304/19f1f45693fb48448cdabdf7f2e822d8.jpeg',
        '6878244-team123380-20260304.jpg',
    ),
    (123380, 6883624): (  # 艾菲尔丁 — 艾菲尔丁
        'https://q7.itc.cn/q_85,c_zoom,w_600/images03/20260304/17bbd787710a4ef990adc926f8b64fd2.jpeg',
        '6883624-team123380-20260304.jpg',
    ),
    (123380, 6892251): (  # 东航 — 东航
        'https://q1.itc.cn/q_85,c_zoom,w_600/images03/20260304/85d89cc2a06746bfb4473dd45ee5e543.jpeg',
        '6892251-team123380-20260304.jpg',
    ),
    (123380, 6936990): (  # 内尔松 — 内尔松-卢斯
        'https://q7.itc.cn/q_85,c_zoom,w_600/images03/20260304/0cf2199a7ac5497bb076117bae960805.jpeg',
        '6936990-team123380-20260304.jpg',
    ),
    (123380, 7476928): (  # 孟敬朝 — 孟敬朝
        'https://q5.itc.cn/q_85,c_zoom,w_600/images03/20260304/feecececcc7644a28486f53079db768e.jpeg',
        '7476928-team123380-20260304.jpg',
    ),
    (123380, 7726315): (  # 李昊 — 李昊
        'https://q3.itc.cn/q_85,c_zoom,w_600/images03/20260304/485648c4fb774486abb1301702015f67.jpeg',
        '7726315-team123380-20260304.jpg',
    ),
    (123380, 7956817): (  # 戚秉宇 — 戚秉宇
        'https://q4.itc.cn/q_85,c_zoom,w_600/images03/20260304/da24a0c36e1e4513820179fdf6f04ad1.jpeg',
        '7956817-team123380-20260304.jpg',
    ),
    (123380, 8343879): (  # 梅米舍维奇 — 梅米舍维奇
        'https://q0.itc.cn/q_85,c_zoom,w_600/images03/20260304/bf4595c10b4b43859bdc9417b7a4e0d0.jpeg',
        '8343879-team123380-20260304.jpg',
    ),
    (123380, 8414275): (  # 巴拉克 — 巴拉克
        'https://q0.itc.cn/q_85,c_zoom,w_600/images03/20260304/08cd7c99640448078562b6dd691d4e7e.jpeg',
        '8414275-team123380-20260304.jpg',
    ),
    (123380, 8421740): (  # 贾伟伟 — 贾伟伟
        'https://q1.itc.cn/q_85,c_zoom,w_600/images03/20260304/58c326a625874806b4faa24cd73f1663.jpeg',
        '8421740-team123380-20260304.jpg',
    ),
    (123380, 8424252): (  # 杨展彭 — 杨展彭
        'https://q8.itc.cn/q_85,c_zoom,w_600/images03/20260304/6b4327ec2d884317845765e22e5bea9c.jpeg',
        '8424252-team123380-20260304.jpg',
    ),
    (123380, 8428327): (  # 刘致成 — 刘致成
        'https://q8.itc.cn/q_85,c_zoom,w_600/images03/20260304/cd1b6ebc66d142aead05c76e79f13520.jpeg',
        '8428327-team123380-20260304.jpg',
    ),
    (123380, 8447219): (  # 雷森德 — 雷森德
        'https://q9.itc.cn/q_85,c_zoom,w_600/images03/20260304/e2be8d992a6f450b885fd9fa791cdb0a.jpeg',
        '8447219-team123380-20260304.jpg',
    ),
    (123380, 8449899): (  # 王庚睿 — 王庚睿
        'https://q0.itc.cn/q_85,c_zoom,w_600/images03/20260304/a15b3dacca3e43a09036560cc35d6889.jpeg',
        '8449899-team123380-20260304.jpg',
    ),
    (123380, 8470259): (  # 王梓权 — 王梓权
        'https://q1.itc.cn/q_85,c_zoom,w_600/images03/20260304/798e3d3d20fa428787a50c7575d68c41.jpeg',
        '8470259-team123380-20260304.jpg',
    ),
}
