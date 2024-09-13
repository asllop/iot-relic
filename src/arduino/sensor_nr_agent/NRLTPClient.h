#ifndef NRLTPClient_h
#define NRLTPClient_h

#include "Arduino.h"

typedef enum HunkType {
  ClientId = 1,
  Timestamp = 2,
  IntMetric = 3,
  FloatMetric = 4
} HunkType;

class NRLTPClient {
  public:
    NRLTPClient();
    bool beginDatagram(char *buffer, unsigned int bufferSize);
    bool addClientIdHunk(char *clientId);
    void addTimestampHunk(unsigned long timestamp);
    bool addIntMetricHunk(long *metrics, int numMetrics, int metricType, char *metricName);
    void addFloatMetricHunk(float *metrics, int numMetrics, int metricType, char *metricName);
    int endDatagram();

  private:
    char *buffer;
    unsigned int bufferSize;
    int datagramIndex;

    bool setHeader(HunkType type, unsigned short bodySize);
    bool setMetricHeader(HunkType type, int metricType, char *metricName, int numMetrics, int metricSize);
};

#endif