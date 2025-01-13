from seleniumwire import webdriver  # Use selenium-wire for network interception
from selenium.webdriver.firefox.service import Service
from selenium.webdriver.firefox.options import Options

# Set path to geckodriver
service = Service('/Applications/geckodriver')  # Ensure the path to geckodriver is correct

# Set Firefox options
options = Options()

# Launch Firefox with Selenium Wire
driver = webdriver.Firefox(service=service, options=options)

# Open the page
driver.get('https://maxroll.gg/poe2/passive-tree/')

# Capture JavaScript file requests
js_files = [request.url for request in driver.requests if request.response and request.url.endswith('.js')]

print('JavaScript files:', js_files)

# Clean up
driver.quit()
