using System;
using System.Collections.Generic;
using System.IO;

namespace EcoNet.BeeSafeAI.ThermalValidator
{
    public class BroodTempRecord
    {
        public DateTime Timestamp { get; set; }
        public double TemperatureC { get; set; }
    }

    public class ThermalValidator
    {
        private const double MinBroodTemp = 33.0;
        private const double MaxBroodTemp = 36.0;
        private const double MaxAllowedDriftHours = 4.0;

        public (bool IsHealthy, double HoursOutsideCorridor, string Recommendation) Validate(List<BroodTempRecord> records)
        {
            double hoursOutside = 0.0;
            foreach (var r in records)
            {
                if (r.TemperatureC < MinBroodTemp || r.TemperatureC > MaxBroodTemp)
                {
                    // Assume 1-hour sampling interval
                    hoursOutside += 1.0;
                }
            }

            bool healthy = hoursOutside <= MaxAllowedDriftHours;
            string recommendation = healthy
                ? "Corridor maintained – no action required."
                : "Thermal stress detected. Recommend passive external shading or improved ventilation (zero-harm only).";

            return (healthy, hoursOutside, recommendation);
        }
    }

    class Program
    {
        static void Main(string[] args)
        {
            // Example usage with CSV input (production systems would stream from sensors)
            var records = new List<BroodTempRecord>
            {
                new() { Timestamp = DateTime.Now, TemperatureC = 34.5 },
                new() { Timestamp = DateTime.Now.AddHours(1), TemperatureC = 37.2 } // Stress event
            };

            var validator = new ThermalValidator();
            var result = validator.Validate(records);
            Console.WriteLine($"Healthy: {result.IsHealthy}");
            Console.WriteLine($"Hours outside 33–36°C: {result.HoursOutsideCorridor}");
            Console.WriteLine(result.Recommendation);
        }
    }
}
