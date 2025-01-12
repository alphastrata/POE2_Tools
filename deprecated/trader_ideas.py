import undetected_chromedriver.v2 as uc
import json
import logging

# Configure logging to include debug prints
logging.basicConfig(level=logging.DEBUG, format='%(asctime)s - %(levelname)s - %(message)s')
logger = logging.getLogger()

# Cookies with the provided values
COOKIES = {
    "cf_clearance": "_X4yBnESiG.3BB4CORaYc.EgD8zShuzhUCwwAz1C28M-1735955539-1.2.1.1-7qpkceOw8QQQJYryPfoEr7kXzxXhQtJ49SKoVKJcT26zQaCdTRB_q8E63m7Ia9l1OFJd9etQoV4zJ4XY8EmiUbcTAAFETBSyoX4gtqEx19ClQ98jQOrW0em9ah.Y.dZj2QSLxzt2ksfpeq6iVWvqbxlkHSATfEJx.7CweBa99Xx440TcZbeSbuMgNKcb5TkzSaIlxHs2AKmZK3fVzCh_8C8KOaJl2osJ7Rrd0N.qV0j0PSGwzXIKNaNH6Vec4sxA4bmMkKfIvvIju3P9rBThd57Cg46UXdUTX3l6JEU51HIp4NAjhR4mX57w8zBmKhMTbHKTPqKrErzvigYqKyI2ZQhXv6CvggSpw7UWdlvwoRW5dS2hxieESfZ1DSPB3nI6knM6brpVV1svXCG_ThpJAA",
    "POESESSID": "95fe908ed0a5cf3c0142cc4ab897efc1"
}

# Base URLs
SEARCH_URL = "https://www.pathofexile.com/api/trade2/search/poe2/Standard"
FETCH_URL = "https://www.pathofexile.com/api/trade2/fetch/"

# Headers to mimic a browser
HEADERS = {
    "User-Agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36",
    "Accept": "application/json, text/javascript, */*; q=0.01",
    "Accept-Language": "en-US,en;q=0.9",
    "Accept-Encoding": "gzip, deflate, br",
    "Connection": "keep-alive",
    "Referer": "https://www.pathofexile.com/trade/search/Standard",
    "Host": "www.pathofexile.com"
}

# Search payload
SEARCH_PAYLOAD = {
    "query": {
        "status": {"option": "online"},
        "stats": [
            {
                "type": "and",
                "filters": [
                    {"id": "explicit.stat_4220027924", "disabled": False},
                    {"id": "explicit.stat_3372524247", "disabled": False}
                ],
                "disabled": False
            }
        ],
        "filters": {
            "req_filters": {
                "filters": {"lvl": {"min": 50, "max": 58}},
                "disabled": False
            },
            "type_filters": {
                "filters": {"category": {"option": "armour.chest"}},
                "disabled": False
            },
            "trade_filters": {
                "filters": {"price": {"min": None, "max": 5, "option": "exalted"}},
                "disabled": False
            },
            "equipment_filters": {
                "filters": {"es": {"min": 200, "max": None}, "ev": {"min": 500, "max": None}},
                "disabled": False
            }
        }
    },
    "sort": {"price": "asc"}
}

def search_items_with_selenium():
    logger.debug("Setting up Selenium WebDriver")
    options = uc.ChromeOptions()
    options.headless = True
    options.add_argument("--disable-gpu")
    options.add_argument("--no-sandbox")

    driver = uc.Chrome(options=options)
    driver.get(SEARCH_URL)

    logger.debug("Setting cookies in Selenium")
    for name, value in COOKIES.items():
        driver.add_cookie({"name": name, "value": value, "domain": "www.pathofexile.com"})

    logger.debug("Refreshing page to apply cookies")
    driver.refresh()

    logger.debug("Sending POST request using JavaScript")
    script = f'''
        return fetch("{SEARCH_URL}", {{
            method: "POST",
            headers: {json.dumps(HEADERS)},
            body: {json.dumps(SEARCH_PAYLOAD)}
        }}).then(res => res.json());
    '''

    try:
        result = driver.execute_script(script)
        logger.info("Search successful: %s", result)
        return result.get("result", [])
    except Exception as e:
        logger.error("Selenium POST request failed: %s", e)
        return None
    finally:
        driver.quit()

if __name__ == "__main__":
    item_hashes = search_items_with_selenium()
    if item_hashes:
        logger.info("Item hashes retrieved: %s", item_hashes)
