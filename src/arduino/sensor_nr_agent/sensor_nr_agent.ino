#include <WiFi101.h>
#include <WiFiUdp.h>
#include <TH02_dev.h>
#include "NRLTPClient.h"
/*
credentials.h must contain the following variables:
const char ssid[] = "YOUR WIFI SSID HERE";
const char pass[] = "YOUR WIFI PASSWORD HERE";
const char server[] = "NRLTP COLLECTOR ADDRESS HERE";
*/
#include "credentials.h"

WiFiUDP Udp;
TH02_dev TH02;
NRLTPClient client;

int status = WL_IDLE_STATUS;
const int datagramBufferSize = 100;
char datagramBuffer[datagramBufferSize];

/// Functions

void setup() {
  Serial.begin(9600);
  while (!Serial);
  Serial.println("Starting...");
  enable_WiFi();
  connect_WiFi();
  printWifiStatus();
  TH02.begin();
  // We are not listening any data, but we have to specify a local port number to call begin.
  Udp.begin(11111);
}

void loop() {
  Serial.print("Sending metrics... ");
  
  Udp.beginPacket(server, 8888);

  client.beginDatagram(datagramBuffer, datagramBufferSize);
  client.addClientIdHunk("ARDUI_01");
  long t = (long)(TH02.ReadTemperature() * 100.0);
  long t_metrics[] = {t};
  client.addIntMetricHunk(t_metrics, 1, 0, "Temp");
  long h = (long)(TH02.ReadHumidity() * 100.0);
  long h_metrics[] = {h};
  client.addIntMetricHunk(h_metrics, 1, 0, "Humi");
  int datagramSize = client.endDatagram();

  Udp.write(datagramBuffer, datagramSize);

  if (Udp.endPacket() > 0) {
    Serial.println("OK.");
  }
  else {
    Serial.println("FAIL.");
  }

  delay(1000);
}

void enable_WiFi() {
  String fv = WiFi.firmwareVersion();
  if (fv < "1.0.0") {
    Serial.println("Please upgrade the firmware");
  }
}

void connect_WiFi() {
  // attempt to connect to Wifi network:
  while (status != WL_CONNECTED) {
    Serial.print("Attempting to connect to SSID: ");
    Serial.println(ssid);

    // Connect to WPA/WPA2 network. Change this line if using open or WEP network:
    status = WiFi.begin(ssid, pass);

    // wait 10 seconds for connection:
    delay(10000);
  }
}

void printWifiStatus() {
  // print the SSID of the network you're attached to:
  Serial.print("SSID: ");
  Serial.println(WiFi.SSID());

  // print your board's IP address:
  IPAddress ip = WiFi.localIP();
  Serial.print("IP Address: ");
  Serial.println(ip);

  // print the received signal strength:

  long rssi = WiFi.RSSI();

  Serial.print("Signal strength (RSSI):");
  Serial.print(rssi);
  Serial.println(" dBm");
}