#include "Arduino.h"
#include "NRLTPClient.h"

NRLTPClient::NRLTPClient() {
    buffer = NULL;
    bufferSize = 0;
    datagramIndex = 0;
}

bool NRLTPClient::beginDatagram(char *buffer, unsigned int bufferSize) {
  datagramIndex = 0;
  this->buffer = buffer;
  this->bufferSize = bufferSize;
  // check that we can fit at least a hunk header and is no greater than the max UDP payload
  return bufferSize >= 8 && bufferSize < 65528;
}

bool NRLTPClient::setHeader(HunkType type, unsigned short bodySize) {
  if (datagramIndex + 8 + bodySize <= bufferSize) {
    // Magic
    buffer[datagramIndex + 0] = 0xAB;
    buffer[datagramIndex + 1] = 0xBC;
    buffer[datagramIndex + 2] = 0xCD;
    // Version
    buffer[datagramIndex + 3] = 1;
    // Type
    buffer[datagramIndex + 4] = (unsigned char)type;
    //TODO: configure endianness depending on the CPU
    // Endianness(LE) + Header size(8)
    buffer[datagramIndex + 5] = 0;
    // Body size
    buffer[datagramIndex + 6] = (unsigned char)(0xFF & bodySize);
    buffer[datagramIndex + 7] = (unsigned char)(0xFF & (bodySize >> 8));

    datagramIndex += 8;
    return true;
  }
  else {
    return false;
  }
}

bool NRLTPClient::addClientIdHunk(char *clientId) {
  int len = strlen(clientId);
  if (datagramIndex + len <= bufferSize) {
    if (!setHeader(HunkType::ClientId, len)) {
      return false;
    }
    // Set Client ID
    memcpy(buffer + datagramIndex, clientId, len);
    datagramIndex += len;
    return true;
  }
  else {
    return false;
  }
}

void NRLTPClient::addTimestampHunk(unsigned long timestamp) {
  //TODO: create a Timestamp hunk in the datagram
}

bool NRLTPClient::setMetricHeader(HunkType type, int metricType, char *metricName, int numMetrics, int metricSize) {
  int len = strlen(metricName);
  if (len > 0 && len <= 32)  {
    int totalSize = len + 1 + numMetrics * metricSize;
    Serial.print("Total size = ");
    Serial.println(totalSize);
    if (setHeader(type, totalSize)) {
      // Set metric type and metric name size
      //TODO: Support metrics of type count. Now metricType is ignored and hardcoded to gauge.
      buffer[datagramIndex + 0] = 0x1F & (len - 1);
      // Set metric name
      memcpy(buffer + datagramIndex + 1, metricName, len);
      datagramIndex += len + 1;
      return true;
    }
    else {
      Serial.println("Error setHeader");
      return false;    
    }
  }
  else {
    Serial.println("Error metric name len");
    return false;
  }
}

bool NRLTPClient::addIntMetricHunk(long *metrics, int numMetrics, int metricType, char *metricName) {
  if (setMetricHeader(HunkType::IntMetric, metricType, metricName, numMetrics, 6)) {
    for (int i = 0 ; i < numMetrics ; i++) {
      // Metric value
      buffer[datagramIndex + 0] = 0xFF & metrics[i];
      buffer[datagramIndex + 1] = 0xFF & (metrics[i] >> 8);
      buffer[datagramIndex + 2] = 0xFF & (metrics[i] >> 16);
      buffer[datagramIndex + 3] = 0xFF & (metrics[i] >> 24);
      // Time offset (TODO: set actual time offset, now hardcoded to 0)
      buffer[datagramIndex + 4] = 0;
      buffer[datagramIndex + 5] = 0;
      datagramIndex += 6;
    }
    return true;
  }
  else {
    Serial.println("Error setMetricHeader");
    return false;
  }
}

void NRLTPClient::addFloatMetricHunk(float *metrics, int numMetrics, int metricType, char *metricName) {
  //TODO: create a Float Metrics hunk in the datagram
}

int NRLTPClient::endDatagram() {
  int size = datagramIndex;
  buffer = NULL;
  bufferSize = 0;
  datagramIndex = 0;
  return size;
}