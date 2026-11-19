-- Seed curated transfer candidates for "Be the Director" (我来当总监).
-- Idempotent upsert keyed by `code`.

INSERT INTO f_i_director_candidates (
    code,
    player_name,
    position,
    country,
    is_foreign,
    league_tag,
    current_club,
    tags,
    target_team_id,
    avatar_url,
    sort_order,
    active
) VALUES
    -- =========================================================================
    -- K League 1 (K1): 21 foreign & Korean/Asian-quota targets
    -- =========================================================================
    ('k1-cesinha', '塞西尼亚', 'FW', '巴西', TRUE, 'K1', '大邱FC', '["K联赛核心","传射兼备"]'::jsonb, 77680, NULL, 10, TRUE),
    ('k1-oberdan', '奥贝尔丹', 'MF', '巴西', TRUE, 'K1', '浦项铁人', '["全能B2B","中场铁闸"]'::jsonb, 77680, NULL, 20, TRUE),
    ('k1-anderson-oliveira', '安德森·奥利维拉', 'FW', '巴西', TRUE, 'K1', '水原FC', '["K联赛助攻王","边路爆点"]'::jsonb, 77680, NULL, 30, TRUE),
    ('k1-stefan-mugosa', '斯特凡·穆戈沙', 'FW', '黑山', TRUE, 'K1', '仁川联', '["K联赛金靴","禁区支点"]'::jsonb, NULL, NULL, 40, TRUE),
    ('k1-stanislav-iljutcenko', '伊柳琴科', 'FW', '德国', TRUE, 'K1', '首尔FC', '["高中锋","头球强势"]'::jsonb, NULL, NULL, 50, TRUE),
    ('k1-jasir-asani', '亚西尔·阿萨尼', 'FW', '阿尔巴尼亚', TRUE, 'K1', '光州FC', '["亚冠杀手","左脚重炮"]'::jsonb, 77680, NULL, 60, TRUE),
    ('k1-yago-cariello', '雅戈·卡列洛', 'FW', '巴西', TRUE, 'K1', '蔚山HD', '["冲击型中锋","门前高效"]'::jsonb, NULL, NULL, 70, TRUE),
    ('k1-darijan-bojanic', '博亚尼奇', 'MF', '瑞典', TRUE, 'K1', '蔚山HD', '["中场大脑","长传调度"]'::jsonb, NULL, NULL, 80, TRUE),
    ('k1-gustav-ludwigson', '路德维格松', 'FW', '瑞典', TRUE, 'K1', '蔚山HD', '["反击快马","跑动积极"]'::jsonb, NULL, NULL, 90, TRUE),
    ('k1-jesse-lingard', '林加德', 'MF', '英格兰', TRUE, 'K1', '首尔FC', '["英超名将","流量担当"]'::jsonb, NULL, NULL, 100, TRUE),
    ('k1-yazan-al-arab', '亚赞·阿拉伯', 'DF', '约旦', TRUE, 'K1', '首尔FC', '["亚外中卫","防空稳健"]'::jsonb, 77680, NULL, 110, TRUE),
    ('k1-hernandes', '埃尔南德斯', 'FW', '巴西', TRUE, 'K1', '全北现代', '["边中皆宜","突破犀利"]'::jsonb, NULL, NULL, 120, TRUE),
    ('k1-tiago-orobo', '蒂亚戈·奥罗博', 'FW', '巴西', TRUE, 'K1', '全北现代', '["抢点中锋","背身拿球"]'::jsonb, NULL, NULL, 130, TRUE),
    ('k1-wanderson', '万德森', 'DF', '巴西', TRUE, 'K1', '浦项铁人', '["攻击边卫","K联赛最佳阵"]'::jsonb, NULL, NULL, 140, TRUE),
    ('k1-gerso-fernandes', '杰尔索', 'FW', '几内亚比绍', TRUE, 'K1', '仁川联', '["边路快马","反击推进"]'::jsonb, NULL, NULL, 150, TRUE),
    ('k1-edgar-bruno', '埃德加', 'FW', '巴西', TRUE, 'K1', '大邱FC', '["高点支点","高空轰炸"]'::jsonb, NULL, NULL, 160, TRUE),
    ('k1-joo-min-kyu', '周敏圭', 'FW', '韩国', TRUE, 'K1', '蔚山HD', '["韩国国脚","本土神锋"]'::jsonb, NULL, NULL, 170, TRUE),
    ('k1-lee-dong-gyeong', '李东炅', 'MF', '韩国', TRUE, 'K1', '金泉尚武', '["韩国国脚","左脚组织"]'::jsonb, 77680, NULL, 180, TRUE),
    ('k1-song-min-kyu', '宋旻奎', 'FW', '韩国', TRUE, 'K1', '全北现代', '["韩国国脚","内切边锋"]'::jsonb, NULL, NULL, 190, TRUE),
    ('k1-um-won-sang', '严元相', 'FW', '韩国', TRUE, 'K1', '蔚山HD', '["韩国快马","边路爆破"]'::jsonb, NULL, NULL, 200, TRUE),
    ('k1-kim-young-gwon', '金英权', 'DF', '韩国', TRUE, 'K1', '蔚山HD', '["韩国国脚","中超旧将","出球中卫"]'::jsonb, NULL, NULL, 210, TRUE),

    -- =========================================================================
    -- K League 2 (K2): 5 value-pick foreign targets
    -- =========================================================================
    ('k2-bruno-mota', '布鲁诺·莫塔', 'FW', '巴西', TRUE, 'K2', '天安城', '["K2金靴级","高性价比"]'::jsonb, NULL, NULL, 220, TRUE),
    ('k2-valdivia', '瓦尔迪维亚', 'MF', '巴西', TRUE, 'K2', '全南天龙', '["K2核心","前腰组织"]'::jsonb, 77680, NULL, 230, TRUE),
    ('k2-luis-mina', '路易斯·米纳', 'FW', '哥伦比亚', TRUE, 'K2', '金浦FC', '["冲击力强","K2高效射手"]'::jsonb, NULL, NULL, 240, TRUE),
    ('k2-matheus-oliveira', '马特乌斯·奥利维拉', 'MF', '巴西', TRUE, 'K2', 'FC安养', '["冲超功臣","传射核心"]'::jsonb, NULL, NULL, 250, TRUE),
    ('k2-rodrigo-bassani', '罗德里戈·巴萨尼', 'MF', '巴西', TRUE, 'K2', '富川FC1995', '["K2大腿","远射精准"]'::jsonb, NULL, NULL, 260, TRUE),

    -- =========================================================================
    -- J1 League (J1): 6 foreign targets
    -- =========================================================================
    ('j1-anderson-lopes', '安德森·洛佩斯', 'FW', '巴西', TRUE, 'J1', '横滨水手', '["J1金靴","门前终结"]'::jsonb, NULL, NULL, 310, TRUE),
    ('j1-leo-ceara', '莱奥·塞阿拉', 'FW', '巴西', TRUE, 'J1', '鹿岛鹿角', '["J1神锋","把握机会强"]'::jsonb, NULL, NULL, 320, TRUE),
    ('j1-matheus-savio', '马特乌斯·萨维奥', 'MF', '巴西', TRUE, 'J1', '浦和红钻', '["J1最佳阵","前场核心"]'::jsonb, 77680, NULL, 330, TRUE),
    ('j1-marcinho', '马尔西尼奥', 'FW', '巴西', TRUE, 'J1', '川崎前锋', '["边路快马","亚冠常客"]'::jsonb, NULL, NULL, 340, TRUE),
    ('j1-rafael-elias', '拉斐尔·埃利亚斯', 'FW', '巴西', TRUE, 'J1', '京都不死鸟', '["高效射手","抢点敏锐"]'::jsonb, NULL, NULL, 350, TRUE),
    ('j1-marius-hoibraten', '霍伊布拉滕', 'DF', '挪威', TRUE, 'J1', '浦和红钻', '["北欧中卫","亚冠冠军防线"]'::jsonb, NULL, NULL, 360, TRUE),

    -- =========================================================================
    -- Brazil (BRA): 5 foreign targets
    -- =========================================================================
    ('bra-erick-pulga', '埃里克·普尔加', 'FW', '巴西', TRUE, 'BRA', '巴伊亚', '["巴乙金靴","边路爆点"]'::jsonb, NULL, NULL, 410, TRUE),
    ('bra-anselmo-ramon', '安塞尔莫·拉蒙', 'FW', '巴西', TRUE, 'BRA', 'CRB', '["中超旧将","支点中锋"]'::jsonb, NULL, NULL, 420, TRUE),
    ('bra-guilherme-augusto', '吉列尔梅', 'FW', '巴西', TRUE, 'BRA', '桑托斯', '["冲甲核心","边路突击"]'::jsonb, NULL, NULL, 430, TRUE),
    ('bra-matheus-peixoto', '马特乌斯·佩肖托', 'FW', '巴西', TRUE, 'BRA', '尤文图德', '["高大中锋","低成本淘宝"]'::jsonb, NULL, NULL, 440, TRUE),
    ('bra-lucas-halter', '卢卡斯·哈尔特', 'DF', '巴西', TRUE, 'BRA', '博塔弗戈', '["巴甲中卫","身体强壮"]'::jsonb, NULL, NULL, 450, TRUE),

    -- =========================================================================
    -- Europe (EUR): 4 foreign targets
    -- =========================================================================
    ('eur-aleksandar-pesic', '佩希奇', 'FW', '塞尔维亚', TRUE, 'EUR', '费伦茨瓦罗斯', '["东欧强锋","欧战经验"]'::jsonb, NULL, NULL, 510, TRUE),
    ('eur-mirko-ivanic', '伊万尼奇', 'MF', '黑山', TRUE, 'EUR', '贝尔格莱德红星', '["技术前腰","欧冠经历"]'::jsonb, NULL, NULL, 520, TRUE),
    ('eur-josip-elez', '埃莱兹', 'DF', '克罗地亚', TRUE, 'EUR', '哈伊杜克', '["出球中卫","克罗地亚防线"]'::jsonb, NULL, NULL, 530, TRUE),
    ('eur-nikola-cumic', '丘米奇', 'FW', '塞尔维亚', TRUE, 'EUR', '喀山红宝石', '["边路突击","反击犀利"]'::jsonb, NULL, NULL, 540, TRUE),

    -- =========================================================================
    -- Free-agent foreign targets (FREE): 3 players
    -- =========================================================================
    ('free-jurgen-locadia', '洛卡迪亚', 'FW', '荷兰', TRUE, 'FREE', '自由身', '["免签即战力","中超熟人"]'::jsonb, NULL, NULL, 610, TRUE),
    ('free-christian-bassogog', '巴索戈', 'FW', '喀麦隆', TRUE, 'FREE', '自由身', '["免签快马","熟悉中超"]'::jsonb, NULL, NULL, 620, TRUE),
    ('free-gabriel-pires', '加布里埃尔·皮雷斯', 'MF', '巴西', TRUE, 'FREE', '自由身', '["免签后腰","拦截出球"]'::jsonb, 77680, NULL, 630, TRUE),

    -- =========================================================================
    -- Domestic targets (CSL / CL1 / FREE): 14 players across GK / DF / MF / FW
    -- =========================================================================
    ('csl-han-jiaqi', '韩佳奇', 'GK', '中国', FALSE, 'CSL', '北京国安', '["国门级别","反应迅捷"]'::jsonb, NULL, NULL, 710, TRUE),
    ('csl-liu-dianzuo', '刘殿座', 'GK', '中国', FALSE, 'FREE', '自由身', '["经验丰富","门线稳健"]'::jsonb, 77680, NULL, 720, TRUE),
    ('csl-wu-shaocong', '吴少聪', 'DF', '中国', FALSE, 'FREE', '自由身', '["留洋回归","左脚中卫"]'::jsonb, 77680, NULL, 730, TRUE),
    ('csl-gao-zhunyi', '高准翼', 'DF', '中国', FALSE, 'CSL', '山东泰山', '["国脚边卫","能中能边"]'::jsonb, NULL, NULL, 740, TRUE),
    ('csl-yang-shuai', '杨帅', 'DF', '中国', FALSE, 'CSL', '河南队', '["正面防守强","性价比高"]'::jsonb, NULL, NULL, 750, TRUE),
    ('csl-li-shuai', '李帅', 'DF', '中国', FALSE, 'CSL', '上海海港', '["左路往返","传中精准"]'::jsonb, NULL, NULL, 760, TRUE),
    ('csl-xie-pengfei', '谢鹏飞', 'MF', '中国', FALSE, 'CSL', '上海申花', '["国脚中场","定位球大师"]'::jsonb, NULL, NULL, 770, TRUE),
    ('csl-dai-weijun', '戴伟浚', 'MF', '中国', FALSE, 'CSL', '上海申花', '["持球推进","即战力"]'::jsonb, 77680, NULL, 780, TRUE),
    ('csl-liao-lisheng', '廖力生', 'MF', '中国', FALSE, 'FREE', '自由身', '["中场节拍器","远射威胁"]'::jsonb, 77680, NULL, 790, TRUE),
    ('cl1-xu-bin', '徐彬', 'MF', '中国', FALSE, 'CL1', '广州队', '["U23新星","拦截覆盖"]'::jsonb, NULL, NULL, 800, TRUE),
    ('csl-lin-liangming', '林良铭', 'FW', '中国', FALSE, 'CSL', '北京国安', '["国脚边锋","内切终结"]'::jsonb, NULL, NULL, 810, TRUE),
    ('csl-xie-wenneng', '谢文能', 'FW', '中国', FALSE, 'CSL', '山东泰山', '["新生代国脚","反击利器"]'::jsonb, NULL, NULL, 820, TRUE),
    ('cl1-xiang-yuwang', '向余望', 'FW', '中国', FALSE, 'CL1', '重庆铜梁龙', '["中甲金靴","潜力前锋"]'::jsonb, 77680, NULL, 830, TRUE),
    ('csl-wang-ziming', '王子铭', 'FW', '中国', FALSE, 'CSL', '北京国安', '["超级替补","门前嗅觉"]'::jsonb, NULL, NULL, 840, TRUE)
ON CONFLICT (code) DO UPDATE SET
    player_name = EXCLUDED.player_name,
    position = EXCLUDED.position,
    country = EXCLUDED.country,
    is_foreign = EXCLUDED.is_foreign,
    league_tag = EXCLUDED.league_tag,
    current_club = EXCLUDED.current_club,
    tags = EXCLUDED.tags,
    target_team_id = EXCLUDED.target_team_id,
    avatar_url = EXCLUDED.avatar_url,
    sort_order = EXCLUDED.sort_order,
    active = EXCLUDED.active,
    updated_at = NOW();
