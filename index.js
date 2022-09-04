async function getLocation() {
    const response = await fetch("/location")
    const body = await response.json()
    return body
}

window.onload = async () => {
    const locPromise = getLocation()
    const colo = document.getElementById("colo")
    const asn = document.getElementById("asn")
    const country = document.getElementById("country")
    const city = document.getElementById("city")
    const continent = document.getElementById("continent")
    const coordinates = document.getElementById("coordinates")
    const postal_code = document.getElementById("postalcode")
    const metro_code = document.getElementById("metrocode")
    const region = document.getElementById("region")
    const region_code = document.getElementById("regioncode")
    const http_version = document.getElementById("httpversion")
    const location = await locPromise
    colo.innerText = location.colo
    asn.innerText = location.asn
    country.innerText = location.country
    city.innerText = location.city
    continent.innerText = location.continent
    coordinates.innerText = location.coordinates
    postal_code.innerText = location.postal_code
    metro_code.innerText = location.metro_code
    region.innerText = location.region
    region_code.innerText = location.region_code
    http_version.innerText = location.http_version
    console.log(location)
}
